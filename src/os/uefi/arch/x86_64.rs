use core::{arch::asm, mem};

use uefi::Result;
use x86_64::{
    PhysAddr,
    registers::{
        control::{Cr0, Cr0Flags, Cr3, Cr4, Cr4Flags},
        model_specific::Efer,
    },
    structures::paging::{PageTableFlags, PhysFrame},
};

use super::super::{OsEfi, memory_map::memory_map};
use crate::{KernelArgs, logger::LOGGER};

unsafe extern "C" fn kernel_entry(
    page_phys: usize,
    stack: u64,
    func: u64,
    args: *const KernelArgs,
) -> ! {
    unsafe {
        // Read memory map and exit boot services
        memory_map().exit_boot_services();

        // Enable FXSAVE/FXRSTOR, Page Global, Page Address Extension, and Page Size
        // Extension
        // x86_64 crate: Cr4::read() returns Cr4Flags
        let mut cr4 = Cr4::read();
        cr4 |= Cr4Flags::OSFXSR
            | Cr4Flags::PAGE_GLOBAL
            | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
            | Cr4Flags::PAGE_SIZE_EXTENSION;
        Cr4::write(cr4);

        // Enable Long mode and NX bit
        // Efer::read() -> EferFlags
        let mut efer = Efer::read();
        efer |= x86_64::registers::model_specific::EferFlags::LONG_MODE_ENABLE
            | x86_64::registers::model_specific::EferFlags::NO_EXECUTE_ENABLE;
        unsafe { Efer::write(efer) };

        // Set new page map
        // Cr3::write(pml4_frame, flags)
        let phys_frame = PhysFrame::containing_address(PhysAddr::new(page_phys as u64));
        Cr3::write(phys_frame, Cr3::read().1);

        // Enable paging, write protect kernel, protected mode
        let mut cr0 = Cr0::read();
        cr0 |= Cr0Flags::PAGING | Cr0Flags::WRITE_PROTECT | Cr0Flags::PROTECTED_MODE_ENABLE;
        Cr0::write(cr0);

        // Set stack
        asm!("mov rsp, {}", in(reg) stack);

        // Call kernel entry
        let entry_fn: extern "sysv64" fn(*const KernelArgs) -> ! = mem::transmute(func);
        entry_fn(args);
    }
}

pub fn main() -> Result<()> {
    crate::logger::LOGGER.init();

    let mut os = OsEfi::new();

    // In uefi 0.28, stdout() returns &mut Output.
    unsafe {
        let _ = os
            .st
            .boot_services()
            .set_image_handle(crate::os::uefi::image_handle());
    }

    // In uefi 0.28, stdout() returns &mut Output.
    // In uefi 0.28, stdout() returns &mut Output.
    let output = os.st.stdout();
    let _ = output.enable_cursor(false);

    let (page_phys, func, args) = crate::ignite_main(&mut os);

    unsafe {
        kernel_entry(
            page_phys,
            args.stack_base
                + args.stack_size
                + if crate::KERNEL_64BIT {
                    crate::arch::x64::PHYS_OFFSET
                } else {
                    crate::arch::x32::PHYS_OFFSET as u64
                },
            func,
            &args,
        );
    }
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}
