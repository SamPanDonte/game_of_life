use std::{collections::VecDeque, sync::Arc, time::Instant};

use egui_winit_vulkano::{egui, Gui};
use vulkano::{image::ImageViewAbstract, sync::GpuFuture};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::{event::WindowEvent, event_loop::EventLoop};

/// This struct represents controls menu.
pub struct Controller {
    gui: Gui,
    speed: u32,
    max_speed: u32,
    pub fps_counter: VecDeque<Instant>,
}

impl Controller {
    /// Create [`Controller`] instance.
    #[inline]
    pub fn new<T>(renderer: &VulkanoWindowRenderer, event_loop: &EventLoop<T>) -> Self {
        let gui = Gui::new(
            event_loop,
            renderer.surface(),
            Some(renderer.swapchain_format()),
            renderer.graphics_queue(),
            true,
        );
        let max_speed = renderer
            .window()
            .available_monitors()
            .next()
            .and_then(|monitor| monitor.refresh_rate_millihertz())
            .unwrap_or(60000)
            / 1000;

        Self {
            gui,
            speed: 60,
            max_speed,
            fps_counter: VecDeque::new(),
        }
    }

    /// Update equvalent of [`Gui`] update method.
    #[inline]
    pub fn update(&mut self, event: &WindowEvent) -> bool {
        self.gui.update(event)
    }

    /// Draw gui on screen
    pub fn draw(
        &mut self,
        future: Box<dyn GpuFuture>,
        image: Arc<dyn ImageViewAbstract>,
    ) -> Box<dyn GpuFuture> {
        self.gui.immediate_ui(|ui| {
            let ctx = ui.context();

            egui::containers::Window::new("Controls").show(&ctx, |ui| {
                ui.label(format!("Frames per second: {}", self.fps_counter.len()));
                ui.add(
                    egui::Slider::new(&mut self.speed, 0..=self.max_speed).text("Simulation speed"),
                );
            });
        });
        self.gui.draw_on_image(future, image)
    }

    /// Returns the speed of the simulation.
    pub fn speed(&self) -> u128 {
        self.speed.into()
    }
}
