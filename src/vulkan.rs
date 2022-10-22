//! This module contains basic [`vulkano_util`] configuration and initialization.
//!
//! It can only be compiled using cargo as it requires
//! environment variables at compile time to be set.
//!
use std::sync::Arc;

use vulkano::{
    buffer::{BufferUsage, DeviceLocalBuffer},
    device::Device,
    instance::{InstanceCreateInfo, InstanceExtensions},
    Version,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::VulkanoWindowRenderer,
    window::WindowDescriptor,
};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::GpuBuffer;

static APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

/// Creates [`VulkanoContext`] with custom values.
///
/// It provides application name and version from `Cargo.toml`.
/// On debug compilation it enables `VK_LAYER_KHRONOS_validation` layer.
/// On macOS it enables `VK_KHR_portability_subset`.
///
/// # Panics
///
/// - when the underlying Vulkano struct creations fail.
/// - when cargo version numbers cannot be parsed.
#[inline]
#[must_use]
pub fn vulkano_context() -> VulkanoContext {
    static PARSE_ERROR: &str = "Cargo version is not valid.";

    VulkanoContext::new(VulkanoConfig {
        instance_create_info: InstanceCreateInfo {
            application_name: Some(APPLICATION_NAME.to_owned()),
            application_version: Version {
                major: env!("CARGO_PKG_VERSION_MAJOR").parse().expect(PARSE_ERROR),
                minor: env!("CARGO_PKG_VERSION_MINOR").parse().expect(PARSE_ERROR),
                patch: env!("CARGO_PKG_VERSION_PATCH").parse().expect(PARSE_ERROR),
            },
            #[cfg(debug_assertions)]
            enabled_layers: vec!["VK_LAYER_KHRONOS_validation".to_string()],
            enabled_extensions: InstanceExtensions::empty(),
            #[cfg(target_os = "macos")]
            enumerate_portability: true,
            ..Default::default()
        },
        ..Default::default()
    })
}

/// Creates [`Window`] with custom values.
///
/// Window title is set to application name from `Cargo.toml`.
///
/// # Panics
///
/// - when window creating returned an error.
#[inline]
fn create_window<T>(event_loop: &EventLoop<T>) -> Window {
    WindowBuilder::default()
        .with_title(APPLICATION_NAME)
        .build(event_loop)
        .expect("Cannot create window with winit")
}

/// Creates [`VulkanoWindowRenderer`] with custom values.
///
/// # Panics
///
/// - when the underlying Vulkano struct creations fail.
#[inline]
#[must_use]
pub fn vulkano_renderer<T>(
    context: &VulkanoContext,
    event_loop: &EventLoop<T>,
) -> VulkanoWindowRenderer {
    VulkanoWindowRenderer::new(
        context,
        create_window(event_loop),
        &WindowDescriptor::default(),
        |_| {},
    )
}

/// Creates [`GpuBuffer`] with custom values.
///
/// Size argument is tuple representing width and height of the buffer.
/// In reality buffer is 1D array of `u32` values with size of size.0 * size.1.
///
/// # Panics
///
/// - when the underlying Vulkano struct creations fail.
#[inline]
#[must_use]
pub fn create_gpu_buffer(
    device: &Arc<Device>,
    size: (u32, u32),
    transfer_src: bool,
) -> Arc<GpuBuffer> {
    DeviceLocalBuffer::array(
        device.clone(),
        u64::from(size.0) * u64::from(size.1),
        BufferUsage {
            storage_buffer: true,
            transfer_dst: true,
            transfer_src,
            ..BufferUsage::empty()
        },
        device.active_queue_family_indices().iter().copied(),
    )
    .expect("Failed to create device local buffer")
}
