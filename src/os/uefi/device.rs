use alloc::{string::String, vec, vec::Vec};
use core::{fmt::Write, mem, slice};

use uefi::{
    Handle, Status,
    proto::{
        Protocol,
        device_path::{DevicePath, DeviceSubType, DeviceType},
        loaded_image::LoadedImage,
        media::{
            file::{File, FileAttribute, FileMode, FileType},
            fs::SimpleFileSystem,
        },
    },
};

use crate::redstonefs::{BLOCK_SIZE, Disk, RECORD_SIZE};

// Standard GetProtocol wrapper
pub fn get_protocol<P: Protocol>(handle: Handle) -> uefi::Result<&'static mut P> {
    let mut sys_tab = unsafe { uefi::helpers::system_table() };
    let bs = sys_tab.boot_services();
    let image = crate::os::uefi::image_handle();

    unsafe {
        use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams};
        let mut protocol = bs.open_protocol::<P>(
            OpenProtocolParams {
                handle,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )?;

        // Prevent ScopedProtocol from dropping
        let ptr = &mut *protocol as *mut P;
        core::mem::forget(protocol);
        Ok(&mut *ptr)
    }
}

use super::disk::{DiskEfi, DiskOrFileEfi};

#[derive(Debug)]
enum DevicePathRelation {
    This,
    Parent(usize),
    Child(usize),
    None,
}

fn device_path_relation(a_path: &DevicePath, b_path: &DevicePath) -> DevicePathRelation {
    let mut a_iter = a_path.node_iter();
    let mut b_iter = b_path.node_iter();

    let mut a_count = 0;
    let mut b_count = 0;

    loop {
        match (a_iter.next(), b_iter.next()) {
            (None, None) => return DevicePathRelation::This,
            (None, Some(_)) => return DevicePathRelation::Parent(b_count),
            (Some(_), None) => return DevicePathRelation::Child(a_count),
            (Some(a_node), Some(b_node)) => {
                if a_node.device_type() != b_node.device_type() {
                    return DevicePathRelation::None;
                }
                if a_node.sub_type() != b_node.sub_type() {
                    return DevicePathRelation::None;
                }
            },
        }
        a_count += 1;
        b_count += 1;
    }
}

unsafe fn device_path_from_ptr(ptr: *mut u8) -> &'static mut DevicePath {
    unsafe {
        let mut len = 0;
        let mut curr = ptr;
        loop {
            let type_code = *curr;
            let subtype = *curr.add(1);
            let length = (*curr.add(3) as u16) << 8 | (*curr.add(2) as u16);

            len += length as usize;
            curr = curr.add(length as usize);

            if type_code == 0x7F && subtype == 0xFF {
                break;
            }
            // Safety check
            if length < 4 {
                break;
            }
        }

        let slice = slice::from_raw_parts_mut(ptr, len);
        mem::transmute(slice)
    }
}

fn esp_live_image(esp_handle: Handle, esp_device_path: &DevicePath) -> Option<Vec<u8>> {
    let mut esp_fs = match get_protocol::<SimpleFileSystem>(esp_handle) {
        Ok(esp_fs) => esp_fs,
        Err(err) => {
            log::warn!("Failed to find SimpleFileSystem protocol: {:?}", err);
            return None;
        },
    };

    let mut root = match esp_fs.open_volume() {
        Ok(root) => root,
        Err(err) => {
            log::warn!("Failed to open ESP filesystem: {:?}", err);
            return None;
        },
    };

    let filename_cstr = uefi::CStr16::from_u16_with_nul(&[
        'r' as u16, 'e' as u16, 'd' as u16, 's' as u16, 't' as u16, 'o' as u16, 'n' as u16,
        'e' as u16, '-' as u16, 'l' as u16, 'i' as u16, 'v' as u16, 'e' as u16, '.' as u16,
        'i' as u16, 's' as u16, 'o' as u16, 0,
    ])
    .unwrap();

    let live_image = match root.open(filename_cstr, FileMode::Read, FileAttribute::empty()) {
        Ok(live_image) => live_image,
        Err(e) if e.status() == Status::NOT_FOUND => return None,
        Err(err) => {
            log::warn!("Failed to open redstone-live.iso: {:?}", err);
            return None;
        },
    };

    let mut file = match live_image.into_regular_file() {
        Some(f) => f,
        None => return None,
    };

    let size = match file.get_boxed_info::<uefi::proto::media::file::FileInfo>() {
        Ok(info) => info.file_size(),
        Err(e) => {
            log::warn!("Failed to get file info: {:?}", e);
            0
        },
    };

    let mut buffer = vec![0u8; size as usize];
    if size > 0 {
        match file.read(&mut buffer) {
            Ok(_) => {},
            Err(e) => panic!("Failed to read live image: {:?}", e),
        }
    }

    Some(buffer)
}

pub struct DiskDevice {
    pub handle:           Handle,
    pub disk:             DiskOrFileEfi,
    pub partition_offset: u64,
    pub device_path:      DevicePathProtocol,
    pub file_path:        Option<&'static str>,
}

// Proxy for DevicePath retrieval
#[repr(C)]
struct DevicePathProxy {
    _private: [u8; 0],
}
unsafe impl uefi::Identify for DevicePathProxy {
    const GUID: uefi::Guid = uefi::proto::device_path::DevicePath::GUID;
}
impl Protocol for DevicePathProxy {}

pub fn disk_device_priority() -> Vec<DiskDevice> {
    let loaded_image = match get_protocol::<LoadedImage>(crate::os::uefi::image_handle()) {
        Ok(loaded_image) => loaded_image,
        Err(err) => {
            log::warn!("Failed to find LoadedImage protocol: {:?}", err);
            return Vec::new();
        },
    };

    // LoadedImage device() returns Handle (in 0.28) or Option<Handle>
    let esp_handle = loaded_image.device();

    // Use expect assuming it's Option<Handle>.
    let esp_handle_val = esp_handle.expect("No device handle");

    let esp_device_path = unsafe {
        let proxy = match get_protocol::<DevicePathProxy>(esp_handle_val) {
            Ok(p) => p,
            Err(err) => {
                log::warn!("Failed to find device path: {:?}", err);
                return Vec::new();
            },
        };
        // Cast to mut pointer
        let ptr = proxy as *mut DevicePathProxy as *mut u8;
        let dp = device_path_from_ptr(ptr);
        DevicePathProtocol(dp)
    };

    if cfg!(feature = "live") {
        if let Some(buffer) = esp_live_image(esp_handle_val, esp_device_path.0) {
            return vec![DiskDevice {
                handle:           esp_handle_val,
                partition_offset: if buffer.len() > 520 && &buffer[512..520] == b"EFI PART" {
                    2 * crate::MIBI as u64
                } else {
                    0
                },
                disk:             DiskOrFileEfi::File(buffer),
                device_path:      esp_device_path,
                file_path:        Some("redstone-live.iso"),
            }];
        }
    }

    let mut sys_tab = unsafe { uefi::helpers::system_table() };
    let bs = sys_tab.boot_services();
    let bio_guid = uefi::proto::media::block::BlockIO::GUID;

    let handles =
        match bs.locate_handle_buffer(uefi::table::boot::SearchType::ByProtocol(&bio_guid)) {
            Ok(h) => h,
            Err(e) => {
                log::warn!("Failed to find block I/O handles: {:?}", e);
                return Vec::new();
            },
        };

    let mut devices = Vec::with_capacity(handles.len());
    for handle in handles.iter() {
        let handle = *handle;

        let disk_proto = match get_protocol::<uefi::proto::media::block::BlockIO>(handle) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if !disk_proto.media().is_media_present() {
            continue;
        }

        let is_partition = disk_proto.media().is_logical_partition();

        let disk = crate::os::uefi::disk::DiskEfi(disk_proto, &mut []);

        let device_path = unsafe {
            let proxy = match get_protocol::<DevicePathProxy>(handle) {
                Ok(p) => p,
                Err(err) => {
                    log::warn!("Failed to find device path: {:?}", err);
                    continue;
                },
            };
            // Cast to mut ptr
            let ptr = proxy as *mut DevicePathProxy as *mut u8;
            let dp = device_path_from_ptr(ptr);
            DevicePathProtocol(dp)
        };

        devices.push(DiskDevice {
            handle,
            partition_offset: if is_partition {
                0
            } else {
                2 * crate::MIBI as u64
            },
            disk: DiskOrFileEfi::Disk(disk),
            device_path,
            file_path: None,
        });
    }

    let mut boot_disks = Vec::with_capacity(1);
    let mut other_disks = Vec::with_capacity(devices.len());

    for d in devices {
        if let DevicePathRelation::Parent(_) =
            device_path_relation(d.device_path.0, esp_device_path.0)
        {
            boot_disks.push(d);
        } else {
            other_disks.push(d);
        }
    }

    let mut all = boot_disks;
    all.append(&mut other_disks);
    all
}

pub fn device_path_to_string(device_path: &DevicePath) -> String {
    let mut s = String::new();
    for node in device_path.node_iter() {
        if !s.is_empty() {
            s.push('/');
        }

        let path_type = node.device_type();
        let sub_type = node.sub_type();

        match path_type {
            DeviceType::HARDWARE => {
                write!(s, "Hardware({:?})", sub_type).unwrap();
            },
            DeviceType::ACPI => {
                write!(s, "Acpi({:?})", sub_type).unwrap();
            },
            DeviceType::MESSAGING => {
                write!(s, "Messaging({:?})", sub_type).unwrap();
            },
            DeviceType::MEDIA => {
                write!(s, "Media({:?})", sub_type).unwrap();
            },
            // Bbs removed or falls into default if present in uefi crate
            DeviceType::END => {
                // End node
            },
            _ => {
                write!(s, "{:?}", path_type).unwrap();
            },
        }
    }
    s
}

// Wrappers
use uefi::Identify;

// We need a wrapper that can hold a reference to DST DevicePath
// But this wrapper itself is Sized (contains reference)
pub struct DevicePathProtocol(pub &'static mut DevicePath);

unsafe impl uefi::Identify for DevicePathProtocol {
    const GUID: uefi::Guid = uefi::proto::device_path::DevicePath::GUID;
}
impl Protocol for DevicePathProtocol {}

pub struct LoadedImageDevicePathProtocol(pub &'static mut DevicePath);

unsafe impl uefi::Identify for LoadedImageDevicePathProtocol {
    const GUID: uefi::Guid = uefi::proto::device_path::DevicePath::GUID;
}
impl Protocol for LoadedImageDevicePathProtocol {}
