use uefi::{
    guid::{GRAPHICS_OUTPUT_PROTOCOL_GUID, Guid},
    proto::{Protocol, console::gop::GraphicsOutput},
}; // Check: 0.28 might be different, but keeping Guid.

pub struct Output(pub &'static mut GraphicsOutput);

use uefi::Identify;

unsafe impl Identify for Output {
    const GUID: Guid = GRAPHICS_OUTPUT_PROTOCOL_GUID;
}

impl Protocol for Output {}

impl Output {
    pub fn new(inner: &'static mut GraphicsOutput) -> Self {
        Output(inner)
    }
}

const EDID_ACTIVE_PROTOCOL_GUID: Guid = Guid(
    0xbd8c1056,
    0x9f36,
    0x44ec,
    [0x92, 0xa8, 0xa6, 0x33, 0x7f, 0x81, 0x79, 0x86],
);

#[allow(non_snake_case)]
#[repr(C)]
pub struct EdidActiveProtocol {
    pub SizeOfEdid: u32,
    pub Edid:       *const u8,
}

pub struct EdidActive(pub &'static mut EdidActiveProtocol);

unsafe impl Identify for EdidActive {
    const GUID: Guid = EDID_ACTIVE_PROTOCOL_GUID;
}

impl Protocol for EdidActive {}

impl EdidActive {
    pub fn new(inner: &'static mut EdidActiveProtocol) -> Self {
        EdidActive(inner)
    }
}
