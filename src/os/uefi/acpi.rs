use core::slice;

use uefi::Guid;
pub const ACPI_20_TABLE_GUID: Guid = uefi::table::cfg::ACPI2_GUID; // Checar se correto, 0.28 provavelmente tem isso em table::cfg
pub const ACPI_TABLE_GUID: Guid = uefi::table::cfg::ACPI_GUID;

use crate::Os;

struct Invalid;

fn validate_rsdp(address: usize, _v2: bool) -> core::result::Result<usize, Invalid> {
    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug)]
    struct Rsdp {
        signature:       [u8; 8], // b"RSD PTR "
        chksum:          u8,
        oem_id:          [u8; 6],
        revision:        u8,
        rsdt_addr:       u32,
        // os campos a seguir estão disponíveis apenas para ACPI 2.0, e são reservados caso
        // contrário
        length:          u32,
        xsdt_addr:       u64,
        extended_chksum: u8,
        _rsvd:           [u8; 3],
    }
    // paginação não está habilitada neste estágio; podemos apenas ler o endereço
    // físico aqui.
    let rsdp_bytes =
        unsafe { core::slice::from_raw_parts(address as *const u8, core::mem::size_of::<Rsdp>()) };
    let rsdp = unsafe {
        (rsdp_bytes.as_ptr() as *const Rsdp)
            .as_ref::<'static>()
            .unwrap()
    };

    log::debug!("RSDP: {:?}", rsdp);

    if rsdp.signature != *b"RSD PTR " {
        return Err(Invalid);
    }
    let mut base_sum = 0u8;
    for base_byte in &rsdp_bytes[..20] {
        base_sum = base_sum.wrapping_add(*base_byte);
    }
    if base_sum != 0 {
        return Err(Invalid);
    }

    if rsdp.revision == 2 {
        let mut extended_sum = 0u8;
        for byte in rsdp_bytes {
            extended_sum = extended_sum.wrapping_add(*byte);
        }

        if extended_sum != 0 {
            return Err(Invalid);
        }
    }

    let length = if rsdp.revision == 2 {
        rsdp.length as usize
    } else {
        core::mem::size_of::<Rsdp>()
    };

    Ok(length)
}

pub(crate) fn find_acpi_table_pointers(os: &impl Os) -> Option<(u64, u64)> {
    let cfg_tables = uefi_services::system_table().config_table();
    let mut acpi = None;
    let mut acpi2 = None;
    for cfg_table in cfg_tables.iter() {
        if cfg_table.guid == ACPI_TABLE_GUID {
            match validate_rsdp(cfg_table.address, false) {
                Ok(len) => {
                    let s =
                        unsafe { core::slice::from_raw_parts(cfg_table.address as *const u8, len) };
                    acpi = Some(s);
                },
                Err(err) => {
                    let length = 36;
                    let bytes = unsafe {
                        core::slice::from_raw_parts(cfg_table.address as *const u8, length)
                    };
                    log::warn!("Invalid ACPI 1.0 Table: {:?} - {:X?}", err, bytes);
                },
            }
        } else if cfg_table.guid == ACPI_20_TABLE_GUID {
            match validate_rsdp(cfg_table.address as usize, true) {
                Ok(len) => {
                    let s =
                        unsafe { core::slice::from_raw_parts(cfg_table.address as *const u8, len) };
                    acpi2 = Some(s);
                },
                Err(err) => {
                    let length = 36;
                    let bytes = unsafe {
                        core::slice::from_raw_parts(cfg_table.address as *const u8, length)
                    };
                    log::warn!("Invalid ACPI 2.0 Table: {:?} - {:X?}", err, bytes);
                },
            }
        }
    }

    let rsdp_area = acpi2.or(acpi).unwrap_or(&[]);

    if !rsdp_area.is_empty() {
        unsafe {
            // Copiar para área alinhada à página
            let size = rsdp_area.len();
            let base = os.alloc_zeroed_page_aligned(size);
            slice::from_raw_parts_mut(base, size).copy_from_slice(rsdp_area);
            Some((base as u64, size as u64))
        }
    } else {
        None
    }
}
