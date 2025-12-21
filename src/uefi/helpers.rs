use crate::uefi::{
    Handle, SystemTable,
    table::{
        boot::{Boot, BootServices},
        system::SystemTable as SystemTableTrait,
    },
};

static mut SYSTEM_TABLE: *mut SystemTable<Boot> = core::ptr::null_mut();
static mut IMAGE_HANDLE: Handle = Handle(core::ptr::null_mut());

pub unsafe fn init() -> crate::uefi::Result<()> {
    // No-op or verification if needed
    // In our manual efi_main, we will set the globals directly via set_system_table
    Ok(())
}

pub unsafe fn set_system_table(st: *mut SystemTable<Boot>, image: Handle) {
    SYSTEM_TABLE = st;
    IMAGE_HANDLE = image;
}

pub unsafe fn system_table() -> SystemTable<Boot> {
    if SYSTEM_TABLE.is_null() {
        panic!("UEFI System Table not initialized");
    }
    // This assumes SystemTable is Copy or we are cloning the wrapper (which is just
    // a pointer wrapper usually) If SystemTable is not Copy/Clone, we might
    // need to return &mut SystemTable or similar. Based on usage in
    // os/uefi/mod.rs, it expects a SystemTable by value.
    core::ptr::read(SYSTEM_TABLE)
}

pub unsafe fn image_handle() -> Handle {
    unsafe { IMAGE_HANDLE }
}
