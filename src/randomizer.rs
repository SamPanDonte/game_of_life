use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Queue,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::GpuFuture,
};

use crate::GpuBuffer;

/// This module contains compiled compute shader and shader data structures.
mod shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/randomizer.comp",
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Pod, Zeroable)]
        }
    }
}

/// This struct represents a pipeline that can be used to
/// randomize the board of the game of life.
pub struct Randomizer {
    group_size: [u32; 3],
    compute_queue: Arc<Queue>,
    pipeline: Arc<ComputePipeline>,
    descriptor: Arc<PersistentDescriptorSet>,
}

impl Randomizer {
    /// Creates a new [`Randomizer`] pipeline.
    ///
    /// It creates new [`ComputePipeline`] and [`PersistentDescriptorSet`].
    ///
    /// # Panics
    ///
    /// - when the underlying Vulkano struct creations fail.
    /// - when the shader entry point is not found.
    /// - when the descriptor set creation fails.
    #[must_use]
    pub fn new(compute_queue: Arc<Queue>, output: Arc<GpuBuffer>, size: (u32, u32)) -> Self {
        let device = compute_queue.device().clone();

        let shader = shader::load(device.clone()).expect("Cannot load compute shader");
        let pipeline = ComputePipeline::new(
            device,
            shader.entry_point("main").expect("Cannot find entry point"),
            &shader::SpecializationConstants {
                width: size.0,
                height: size.1,
            },
            None,
            |_| {},
        )
        .expect("Cannot create compute pipeline");

        let layout = pipeline
            .layout()
            .set_layouts()
            .get(0)
            .expect("Cannot get descriptor set layout");

        let descriptor =
            PersistentDescriptorSet::new(layout.clone(), [WriteDescriptorSet::buffer(0, output)])
                .expect("Cannot create descriptor set");

        let mut group_size = [size.0 / 32, size.1 / 32, 1];
        if size.0 % 32 != 0 {
            group_size[0] += 1;
        }
        if size.1 % 32 != 0 {
            group_size[1] += 1;
        }

        Self {
            group_size,
            compute_queue,
            pipeline,
            descriptor,
        }
    }

    /// Returns a gpu future that can be used to execute the pipeline.
    /// The future will be executed on the compute queue.
    ///
    /// # Panics
    ///
    /// - when the command buffer creation fails.
    /// - when the command buffer building fails.
    /// - when the command buffer recording fails.
    /// - when the command buffer execution fails.
    #[must_use]
    pub fn run(&self) -> Box<dyn GpuFuture> {
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
                shader::ty::PushConstants { seed: rand::random() },
            )
            .bind_pipeline_compute(self.pipeline.clone())
            .dispatch(self.group_size)
            .expect("Cannot record command buffer");

        builder
            .build()
            .expect("Cannot build command buffer")
            .execute(self.compute_queue.clone())
            .expect("Cannot execute command buffer")
            .boxed()
    }
}
