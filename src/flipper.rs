use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Queue,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::GpuFuture,
};

use crate::GpuBuffer;

mod shader {
    vulkano_shaders::shader! {
        path: "src/shaders/flip.comp",
        ty: "compute",
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Pod, Zeroable)]
        }
    }
}

/// This struct represents a pipeline that can be used to
/// flip cells in the game of life.
pub struct Flipper {
    compute_queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    descriptor: Arc<PersistentDescriptorSet>,
}

impl Flipper {
    /// Creates a new [`Flipper`] pipeline.
    /// It creates new [`ComputePipeline`] and [`PersistentDescriptorSet`].
    /// 
    /// # Panics
    /// 
    /// - when the underlying Vulkano struct creations fail.
    /// - when the shader entry point is not found.
    /// - when the descriptor set creation fails.
    /// - when the compute pipeline creation fails.
    #[must_use]
    pub fn new(compute_queue: Arc<Queue>, buffer: Arc<GpuBuffer>, size: (u32, u32)) -> Self {
        let device = compute_queue.device().clone();

        let shader = shader::load(device.clone()).expect("failed to create shader module");
        let pipeline = ComputePipeline::new(
            device,
            shader
                .entry_point("main")
                .expect("failed to find entry point"),
            &shader::SpecializationConstants {
                width: size.0,
                height: size.1,
            },
            None,
            |_| {},
        )
        .expect("failed to create compute pipeline");

        let descriptor = PersistentDescriptorSet::new(
            pipeline
                .layout()
                .set_layouts()
                .get(0)
                .expect("Cannot get descriptor set layout")
                .clone(),
            [WriteDescriptorSet::buffer(0, buffer)],
        )
        .expect("Cannot create descriptor set");

        Self {
            compute_queue,
            pipeline,
            descriptor,
        }
    }

    /// Runs the pipeline and returns gpu future.
    /// As a result cells are flipped.
    /// 
    /// # Panics
    /// 
    /// - when the command buffer creation fails.
    /// - when the command buffer building fails.
    /// - when the command buffer recording fails.
    /// - when the command buffer submission fails.
    #[must_use]
    pub fn flip(&self, position: [u32; 2]) -> Box<dyn GpuFuture> {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.compute_queue.device().clone(),
            self.compute_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Cannot create command buffer builder");

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                self.descriptor.clone(),
            )
            .push_constants(
                self.pipeline.layout().clone(),
                0,
                shader::ty::PushConstants { position },
            )
            .bind_pipeline_compute(self.pipeline.clone())
            .dispatch([1, 1, 1])
            .expect("Cannot record command buffer");

        builder
            .build()
            .expect("Cannot build command buffer")
            .execute(self.compute_queue.clone())
            .expect("Cannot execute command buffer")
            .then_signal_fence_and_flush()
            .expect("Cannot flush command buffer")
            .boxed()
    }
}
