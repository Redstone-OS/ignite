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
        // uefi_services::println!("[DEBUG] kernel_entry: Saindo de Boot Services...");

        // Debug ANTES de exit_boot_services
        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[PRE-EXIT-BS] Calling exit_boot_services...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Read memory map and exit boot services
        memory_map().exit_boot_services();

        // Debug IMEDIATAMENTE APOS exit_boot_services
        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-EXIT-BS] Returned from exit_boot_services!\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // IMPORTANTE: Apos exit_boot_services(), nao podemos mais usar
        // uefi_services::println! porque os servicos UEFI foram desligados.
        // Qualquer tentativa de usar println aqui causara um travamento.

        // Debug via serial direta (sem UEFI)
        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] CR4...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Enable FXSAVE/FXRSTOR, Page Global, Page Address Extension, and Page Size
        // Extension
        let mut cr4 = Cr4::read();
        cr4 |= Cr4Flags::OSFXSR
            | Cr4Flags::PAGE_GLOBAL
            | Cr4Flags::PHYSICAL_ADDRESS_EXTENSION
            | Cr4Flags::PAGE_SIZE_EXTENSION;
        Cr4::write(cr4);

        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] EFER...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Enable Long mode and NX bit
        let mut efer = Efer::read();
        efer |= x86_64::registers::model_specific::EferFlags::LONG_MODE_ENABLE
            | x86_64::registers::model_specific::EferFlags::NO_EXECUTE_ENABLE;
        unsafe { Efer::write(efer) };

        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] CR3...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Set new page map
        let phys_frame = PhysFrame::containing_address(PhysAddr::new(page_phys as u64));
        Cr3::write(phys_frame, Cr3::read().1);

        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] CR0...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Enable paging, write protect kernel, protected mode
        let mut cr0 = Cr0::read();
        cr0 |= Cr0Flags::PAGING | Cr0Flags::WRITE_PROTECT | Cr0Flags::PROTECTED_MODE_ENABLE;
        Cr0::write(cr0);

        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] Stack...\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

        // Set stack
        asm!("mov rsp, {}", in(reg) stack);

        unsafe {
            let port: u16 = 0x3F8;
            for &byte in b"[POST-BS] Jump!\r\n".iter() {
                ::core::arch::asm!("out dx, al", in("dx") port, in("al") byte);
            }
        }

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

    // uefi_services::println!("[DEBUG] Preparando para saltar para o kernel...");
    // uefi_services::println!("[DEBUG] page_phys=0x{:X}, func=0x{:X}", page_phys,
    // func);
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
