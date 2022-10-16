#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
mod config;
mod controller;
mod presenter;
mod simulation;
pub mod vulkan;

use std::time::Instant;

pub use config::*;
pub use controller::*;

use presenter::Presenter;
use rand::Rng;
use simulation::Simulation;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer},
    command_buffer::{
        pool::standard::StandardCommandPoolAlloc, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferInfo, PrimaryAutoCommandBuffer,
    },
    memory::pool::{PotentialDedicatedAllocation, StandardMemoryPoolAlloc},
    sync::{self, GpuFuture},
};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

type GpuBuffer = DeviceLocalBuffer<[u32], PotentialDedicatedAllocation<StandardMemoryPoolAlloc>>;
type CommandBuffer = PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>;

/// This struct represents the game of life.
/// It contains the event loop, renderer, simulationm, controller and the presenter.
pub struct GameOfLife {
    event_loop: EventLoop<()>,
    renderer: VulkanoWindowRenderer,
    simulation: Simulation,
    presenter: Presenter,
    controller: Controller,
}

impl GameOfLife {
    /// Creates a new [`GameOfLife`] instance.
    ///
    /// # Arguments
    ///
    /// - `config` - The configuration for the simulation.
    ///
    /// # Panics
    ///
    /// - when the renderer fails to initialize.
    /// - when the simulation fails to initialize.
    /// - when the presenter fails to initialize.
    /// - when vulkan fails to create any of structures.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let context = vulkan::vulkano_context();
        let event_loop = EventLoop::new();
        let renderer = vulkan::vulkano_renderer(&context, &event_loop);
        let controller = Controller::new(&renderer, &event_loop);
        let buffer = vulkan::create_gpu_buffer(context.device(), config.size(), true);
        let simulation = Simulation::new(renderer.compute_queue(), buffer.clone(), config.size());
        let presenter = Presenter::new(&renderer, buffer.clone(), config.size());

        let mut statwe = [32.0, 3213.0];

        rand::thread_rng().fill(&mut statwe);

        let tmp_buffer = CpuAccessibleBuffer::from_iter(
            context.device().clone(),
            BufferUsage {
                storage_buffer: true,
                transfer_src: true,
                ..BufferUsage::empty()
            },
            false,
            (0..(config.size().0 * config.size().1) as u32)
                .into_iter()
                .map(|_| rand::random::<u32>() % 2),
        )
        .expect("failed to create buffer");

        let mut builder = AutoCommandBufferBuilder::primary(
            context.device().clone(),
            renderer.compute_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("failed to create command buffer");

        builder
            .copy_buffer(CopyBufferInfo::buffers(tmp_buffer, buffer))
            .expect("failed to copy buffer");

        let command_buffer = builder.build().expect("failed to build command buffer");

        sync::now(context.device().clone())
            .then_execute(renderer.compute_queue(), command_buffer)
            .expect("failed to execute command buffer")
            .then_signal_fence_and_flush()
            .expect("failed to execute command buffer")
            .wait(None)
            .expect("failed to wait for command buffer");

        Self {
            event_loop,
            renderer,
            simulation,
            presenter,
            controller,
        }
    }

    /// Runs the Conway's Game of Life simulation.
    ///
    /// # Panics
    ///
    /// - when vulkan fails to create any of structures.
    /// - when vulkan fails to execute any of commands.
    /// - when vulkan fails to wait for any of commands.
    /// - when vulkan fails to present any of frames.
    /// - when vulkan fails to acquire any of frames.
    pub fn run(mut self) -> ! {
        let mut timer = Instant::now();

        self.event_loop.run(move |event, _, flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => flow.set_exit(),
            Event::WindowEvent { event, .. } => {
                self.controller.update(&event);
            }
            Event::MainEventsCleared => {
                let mut future = match self.renderer.acquire() {
                    Ok(future) => future,
                    Err(_) => return,
                };

                let now = Instant::now();
                self.controller.fps_counter.push_back(now);

                while let Some(x) = self.controller.fps_counter.pop_front() {
                    if now - x < std::time::Duration::from_millis(1000) {
                        self.controller.fps_counter.push_front(x);
                        break;
                    }
                }

                let speed = self.controller.speed();
                if speed > 0 && (now - timer).as_millis() > 1000 / speed {
                    timer = now;
                    future = self.simulation.step(future);
                }

                let x = self.presenter.draw(&self.renderer);

                future = future
                    .then_execute(self.renderer.graphics_queue(), x)
                    .expect("failed to execute command buffer")
                    .boxed();

                future = self
                    .controller
                    .draw(future, self.renderer.swapchain_image_view());

                self.renderer.present(future, true);
            }
            _ => (),
        });
    }
}
