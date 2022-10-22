use std::sync::Arc;

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            render_pass::PipelineRenderPassType,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, Subpass},
};
use vulkano_util::renderer::VulkanoWindowRenderer;
use winit::event::WindowEvent;

use crate::{Camera, CommandBuffer, GpuBuffer};

/// This module contains compiled vertex and fragment shaders and shader data structures.
mod shader {
    vulkano_shaders::shader! {
        shaders: {
            vertex: {
                ty: "vertex",
                path: "src/shaders/presenter.vert",
            },
            fragment: {
                ty: "fragment",
                path: "src/shaders/presenter.frag",
            }
        },
        types_meta: {
            use bytemuck::{Pod, Zeroable};
            #[derive(Clone, Copy, Pod, Zeroable)]
        }
    }
}

/// This struct represents a pipeline that can be used to
/// present the game of life.
pub struct Presenter {
    camera: Camera,
    pipeline: Arc<GraphicsPipeline>,
    descriptor: Arc<PersistentDescriptorSet>,
}

impl Presenter {
    /// Creates a new [`Presenter`] pipeline.
    ///
    /// It creates new [`GraphicsPipeline`] and [`PersistentDescriptorSet`].
    ///
    /// # Panics
    ///
    /// - when the underlying Vulkano struct creations fail.
    /// - when the shader entry point is not found.
    /// - when the descriptor set creation fails.
    /// - when the pipeline creation fails.
    /// - when the pipeline layout creation fails.
    #[must_use]
    pub fn new(renderer: &VulkanoWindowRenderer, buffer: Arc<GpuBuffer>, size: (u32, u32)) -> Self {
        let device = renderer.graphics_queue().device().clone();

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: renderer.swapchain_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .expect("Cannot create render pass");
        let subpass = Subpass::from(render_pass, 0).expect("Cannot create subpass");

        let vs = shader::load_vertex(device.clone()).expect("Cannot load vertex shader");
        let fs = shader::load_fragment(device.clone()).expect("Cannot load fragment shader");
        let pipeline = GraphicsPipeline::start()
            .render_pass(subpass)
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleStrip),
            )
            .vertex_shader(
                vs.entry_point("main").expect("Cannot find entry point"),
                (),
            )
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                fs.entry_point("main").expect("Cannot find entry point"),
                shader::FragmentSpecializationConstants {
                    width: size.0,
                    height: size.1,
                },
            )
            .build(device)
            .expect("Cannot create graphics pipeline");

        let layout = pipeline
            .layout()
            .set_layouts()
            .get(0)
            .expect("Cannot get descriptor set layout");
        let descriptor =
            PersistentDescriptorSet::new(layout.clone(), [WriteDescriptorSet::buffer(0, buffer)])
                .expect("Cannot create descriptor set");

        Self {
            camera: Camera::new(size, renderer.window().inner_size()),
            pipeline,
            descriptor,
        }
    }

    /// Updates the camera.
    pub fn update(&mut self, event: &WindowEvent) {
        self.camera.update(event);
    }

    /// Creates a new [`PrimaryAutoCommandBuffer`] that can be used to
    /// present the game of life.
    ///
    /// # Panics
    ///
    /// - when the command buffer creation fails.
    /// - when the framebuffer creation fails.
    /// - when the command buffer recording fails.
    /// - when the command buffer builder creation fails.
    /// - when the render pass begin fails.
    /// - when the command buffer execution fails.
    /// - when the render pass end fails.
    #[must_use]
    pub fn draw(&self, renderer: &VulkanoWindowRenderer, draw_grid: bool) -> CommandBuffer {
        let render_pass = match self.pipeline.render_pass() {
            PipelineRenderPassType::BeginRenderPass(value) => value.render_pass(),
            PipelineRenderPassType::BeginRendering(_) => unreachable!(),
        };

        let framebuffer = Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![renderer.swapchain_image_view()],
                ..Default::default()
            },
        )
        .expect("Failed to create framebuffer");

        let mut builder = AutoCommandBufferBuilder::primary(
            self.pipeline.device().clone(),
            renderer.graphics_queue().queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("Failed to create command buffer builder");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.5, 0.5, 0.5, 0.5].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::Inline,
            )
            .expect("Failed to begin render pass")
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: renderer.surface().window().inner_size().into(),
                    depth_range: 0.0..1.0,
                }],
            )
            .push_constants(
                self.pipeline.layout().clone(),
                0,
                shader::ty::Camera {
                    matrix: self.camera.matrix().to_cols_array_2d(),
                    drawGrid: draw_grid.into(),
                    position: self.camera.cursor_game_position(),
                    _dummy0: [0; 4],
                },
            )
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                self.descriptor.clone(),
            )
            .draw(4, 1, 0, 0)
            .expect("Failed to draw")
            .end_render_pass()
            .expect("Failed to end render pass");

        builder.build().expect("Failed to build command buffer")
    }

    /// Returns the camera.
    #[inline]
    #[must_use]
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}
