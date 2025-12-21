use uefi::{
    Guid, Identify,
    proto::{Protocol, console::gop::GraphicsOutput},
};

pub struct Output(pub &'static mut GraphicsOutput);

unsafe impl Identify for Output {
    const GUID: Guid = uefi::proto::console::gop::GraphicsOutput::GUID;
}

impl Protocol for Output {}

impl Output {
    pub fn new(inner: &'static mut GraphicsOutput) -> Self {
        Output(inner)
    }
}

// Manually verify GUID if needed, but using Protocol's GUID is safer.
// If imported correctly.
const EDID_ACTIVE_PROTOCOL_GUID_BYTES: [u8; 16] = [
    0x56, 0x10, 0x8c, 0xbd, 0x36, 0x9f, 0xec, 0x44, 0x92, 0xa8, 0xa6, 0x33, 0x7f, 0x81, 0x79, 0x86,
];

const EDID_ACTIVE_PROTOCOL_GUID: Guid = Guid::from_bytes(EDID_ACTIVE_PROTOCOL_GUID_BYTES);

#[allow(non_snake_case)]
#[repr(C)]
pub struct EdidActiveProtocol {
    pub SizeOfEdid: u32,
    pub Edid:       *const u8,
}

unsafe impl uefi::Identify for EdidActiveProtocol {
    const GUID: uefi::Guid = EDID_ACTIVE_PROTOCOL_GUID;
}

impl uefi::proto::Protocol for EdidActiveProtocol {}

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
