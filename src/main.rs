//! Bootloader Ignite - Entry Point
//!
//! Este é o ponto de entrada do bootloader. Toda a lógica foi movida para lib.rs.

#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn main(image_handle: Handle, system_table: SystemTable<Boot>) -> Status {
    // Chamar função principal do bootloader (nunca retorna)
    ignite::boot(image_handle, system_table);
}
