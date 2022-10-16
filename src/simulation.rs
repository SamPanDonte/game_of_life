use std::sync::Arc;

use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::Queue,
    pipeline::{ComputePipeline, Pipeline, PipelineBindPoint},
    sync::{self, GpuFuture},
};

use crate::{vulkan, CommandBuffer, GpuBuffer};

/// This module contains compiled compute shader and shader data structures.
mod shader {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/simulation.comp",
    }
}

/// This struct represents a pipeline that can be used to
/// compute the next generation of the game of life.
pub struct Simulation {
    compute_queue: Arc<Queue>,
    main_buffer: Arc<CommandBuffer>,
    copy_buffer: Arc<CommandBuffer>,
}

impl Simulation {
    /// Creates a new [`Simulation`] pipeline.
    ///
    /// It creates new [`GpuBuffer`], [`ComputePipeline`] and [`PersistentDescriptorSet`].
    /// Then it records a command buffer that can be used to execute the pipeline.
    ///
    /// # Panics
    ///
    /// - when the underlying Vulkano struct creations fail.
    /// - when the shader entry point is not found.
    /// - when the descriptor set creation fails.
    /// - when the command buffer creation fails.
    /// - when the command buffer building fails.
    /// - when the command buffer recording fails.
    pub fn new(compute_queue: Arc<Queue>, output: Arc<GpuBuffer>, size: (u32, u32)) -> Self {
        let device = compute_queue.device().clone();
        let input = vulkan::create_gpu_buffer(&device, size, false);

        let shader = shader::load(device.clone()).expect("Cannot load compute shader");
        let pipeline = ComputePipeline::new(
            device.clone(),
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

        let descriptor = PersistentDescriptorSet::new(
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, output.clone()),
                WriteDescriptorSet::buffer(1, input.clone()),
            ],
        )
        .expect("Cannot create descriptor set");

        let mut group_size = [size.0 / 32, size.1 / 32, 1];
        if size.0 % 32 != 0 {
            group_size[0] += 1;
        }
        if size.1 % 32 != 0 {
            group_size[1] += 1;
        }

        let mut builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            compute_queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
        )
        .expect("Cannot create command buffer builder");

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                descriptor,
            )
            .bind_pipeline_compute(pipeline)
            .dispatch(group_size)
            .expect("Cannot record command buffer");

        let main_buffer = Arc::new(builder.build().expect("Cannot build command buffer"));

        let mut builder = AutoCommandBufferBuilder::primary(
            device,
            compute_queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )
        .expect("Cannot create command buffer builder");

        builder
            .copy_buffer(CopyBufferInfo::buffers(output, input))
            .expect("Cannot copy buffer");

        let copy_buffer = Arc::new(builder.build().expect("Cannot build command buffer"));

        Self {
            compute_queue,
            main_buffer,
            copy_buffer,
        }
    }

    /// Executes the pipeline after given [`GpuFuture`].
    /// Returns a new [`GpuFuture`] that can be used to wait for the pipeline to finish.
    /// After the pipeline is finished, simulation of the next generation is ready.
    ///
    /// # Panics
    ///
    /// - when the command buffer submission fails.
    /// - when the command buffer copy fails.
    pub fn step(&self, future: Box<dyn GpuFuture>) -> Box<dyn GpuFuture> {
        future
            .then_execute(self.compute_queue.clone(), self.copy_buffer.clone())
            .expect("Cannot execute command buffer")
            .then_signal_fence_and_flush()
            .expect("Cannot flush command buffer")
            .wait(None)
            .expect("Cannot wait for command buffer");
        sync::now(self.compute_queue.device().clone())
            .then_execute(self.compute_queue.clone(), self.main_buffer.clone())
            .expect("Cannot execute command buffer")
            .then_signal_semaphore_and_flush()
            .expect("Cannot flush command buffer")
            .boxed()
    }
}
