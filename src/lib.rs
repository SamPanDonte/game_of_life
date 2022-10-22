#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::undocumented_unsafe_blocks)]
mod camera;
mod config;
mod controller;
mod presenter;
mod randomizer;
mod simulation;
pub mod vulkan;

pub use camera::*;
pub use config::*;
pub use controller::*;
pub use presenter::*;
pub use randomizer::*;
pub use simulation::*;

use std::time::Instant;

use vulkano::{
    buffer::DeviceLocalBuffer,
    command_buffer::{pool::standard::StandardCommandPoolAlloc, PrimaryAutoCommandBuffer},
    memory::pool::{PotentialDedicatedAllocation, StandardMemoryPoolAlloc},
    sync::GpuFuture,
};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState},
    event_loop::{EventLoop, EventLoopBuilder},
};

type GpuBuffer = DeviceLocalBuffer<[u32], PotentialDedicatedAllocation<StandardMemoryPoolAlloc>>;
type CommandBuffer = PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>;

#[derive(Debug)]
pub enum Message {
    Randomize,
    Clear,
}

/// This struct represents the game of life.
/// It contains the event loop, renderer, simulation, controller and the presenter.
pub struct GameOfLife {
    event_loop: EventLoop<Message>,
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
        let event_loop = EventLoopBuilder::<Message>::with_user_event().build();
        let renderer = vulkan::vulkano_renderer(&context, &event_loop);
        let controller = Controller::new(&renderer, &event_loop);
        let buffer = vulkan::create_gpu_buffer(context.device(), config.size(), true);
        let simulation = Simulation::new(renderer.compute_queue(), buffer.clone(), config.size());
        let presenter = Presenter::new(&renderer, buffer, config.size());

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
        let mut minimized = false;
        let mut flip = false;

        self.event_loop.run(move |event, _, flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => flow.set_exit(),
            Event::WindowEvent { event, .. } => {
                if self.controller.update(&event) {
                    return;
                }
                self.presenter.update(&event);
                if let WindowEvent::Resized(size) = event {
                    if size.height == 0 || size.width == 0 {
                        flow.set_wait();
                        minimized = true;
                    } else {
                        flow.set_poll();
                        minimized = false;
                    }
                }
                if let WindowEvent::MouseInput { state, button, .. } = event {
                    if MouseButton::Right == button && state == ElementState::Pressed {
                        flip = true;
                    }
                }
            }
            Event::UserEvent(Message::Randomize) => {
                self.simulation
                    .randomize()
                    .then_signal_fence_and_flush()
                    .expect("failed to execute command buffer")
                    .wait(None)
                    .expect("failed to wait for command buffer");
            }
            Event::UserEvent(Message::Clear) => {
                self.simulation
                    .clear()
                    .then_signal_fence_and_flush()
                    .expect("failed to execute command buffer")
                    .wait(None)
                    .expect("failed to wait for command buffer");
            }
            Event::MainEventsCleared => {
                if minimized {
                    return;
                }
                let mut future = match self.renderer.acquire() {
                    Ok(future) => future,
                    Err(_) => return,
                };

                let now = Instant::now();
                self.controller.fps_counter.push_back(now);

                while let Some(x) = self.controller.fps_counter.pop_front() {
                    if (now - x).as_millis() < 1000 {
                        self.controller.fps_counter.push_front(x);
                        break;
                    }
                }

                let speed = self.controller.speed();
                if speed > 0 && (now - timer).as_millis() > 1000 / speed {
                    timer = now;
                    future = self.simulation.step(future);
                }
                let x = self
                    .presenter
                    .draw(&self.renderer, self.controller.grid(), flip);

                future = future
                    .then_execute(self.renderer.graphics_queue(), x)
                    .expect("failed to execute command buffer")
                    .boxed();

                future = self
                    .controller
                    .draw(future, self.renderer.swapchain_image_view());

                self.renderer.present(future, true);
                flip = false;
            }
            _ => (),
        });
    }
}
