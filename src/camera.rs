use glam::{Mat4, Vec3};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
};

static SCALE_FACTOR: f32 = 0.1;

/// Struct that represents a camera.
pub struct Camera {
    scale: f32,
    ratio: f32,
    moving: bool,
    game_ratio: f64,
    translation: Vec3,
    screen_size: (f64, f64),
    cursor_pos: PhysicalPosition<f64>,
}

impl Camera {
    /// Creates a new camera.
    #[must_use]
    pub fn new(game_size: (u32, u32), screen_size: PhysicalSize<u32>) -> Self {
        let game_ratio = f64::from(game_size.0) / f64::from(game_size.1);
        let screen_size = (f64::from(screen_size.width), f64::from(screen_size.height));
        let screen_ratio = screen_size.0 / screen_size.1;

        #[allow(clippy::cast_possible_truncation)]
        let ratio = (game_ratio / screen_ratio) as f32;

        Self {
            scale: 1.0,
            ratio,
            moving: false,
            game_ratio,
            translation: Vec3::ZERO,
            screen_size,
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
        }
    }

    /// Updates the camera.
    pub fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(screen_size) => {
                #[allow(clippy::cast_possible_truncation)]
                if screen_size.height != 0 && screen_size.width != 0 {
                    self.screen_size =
                        (f64::from(screen_size.width), f64::from(screen_size.height));
                    let screen_ratio = self.screen_size.0 / self.screen_size.1;
                    self.ratio = (self.game_ratio / screen_ratio) as f32;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                #[allow(clippy::cast_possible_truncation)]
                if self.moving {
                    let dx = (position.x - self.cursor_pos.x) * 2.0 / self.screen_size.0;
                    let dy = (position.y - self.cursor_pos.y) * 2.0 / self.screen_size.1;
                    self.translation.x += dx as f32 / self.scale;
                    self.translation.y += dy as f32 / (self.scale / self.ratio);
                    self.translation.x = self.translation.x.clamp(-1.0, 1.0);
                    self.translation.y = self.translation.y.clamp(-1.0, 1.0);
                }
                self.cursor_pos = *position;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, dy) => {
                        self.scale += dy * SCALE_FACTOR * self.scale;
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.scale += delta.y.signum() as f32 * SCALE_FACTOR * self.scale;
                    }
                }
                self.scale = self.scale.clamp(0.5, 10.0);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    self.moving = *state == ElementState::Pressed;
                }
            }
            _ => (),
        }
    }

    /// Returns the view matrix.
    #[must_use]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale(Vec3::new(self.scale, self.scale / self.ratio, 1.0))
            * Mat4::from_translation(self.translation)
    }
}
