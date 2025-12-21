//! Intrínsecos para tratamento de pânico

use core::{alloc::Layout, arch::asm, panic::PanicInfo};

/// Necessário para lidar com pânicos
#[panic_handler]
pub fn rust_begin_unwind(info: &PanicInfo) -> ! {
    unsafe {
        println!("BOOTLOADER PANIC:\n{}", info);
        loop {
            asm!("hlt");
        }
    }
}
