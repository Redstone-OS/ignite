//! Protocolos UEFI

pub mod console;
pub mod loaded_image;
pub mod media;

pub use console::gop::GRAPHICS_OUTPUT_PROTOCOL_GUID;
pub use loaded_image::LOADED_IMAGE_PROTOCOL_GUID;
pub use media::{file::FILE_PROTOCOL_REVISION, fs::SIMPLE_FILE_SYSTEM_PROTOCOL_GUID};
