// Copyright 2023-2025, [rust@localhost] $ (@3532340532)
//
// This file is part of EfficientScheduler.
//
// EfficientScheduler is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// EfficientScheduler is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along
// with EfficientScheduler. If not, see <https://www.gnu.org/licenses/>.

#![allow(dead_code)]
#![allow(clippy::nursery, clippy::pedantic)]

use std::{ffi::CString, fs, os::unix::fs::PermissionsExt, ptr};

use anyhow::Result;
use libc::{MS_BIND, MS_REC, mount, umount, umount2};
use serde::Deserialize;

#[derive(Deserialize)]
struct Cpuctl {
    bg_uclamp: Uclamp,
    ta_uclamp: Uclamp,
    fg_uclamp: Uclamp,
}

#[derive(Deserialize)]
struct Uclamp {
    max: usize,
    min: usize,
}

pub struct Buffer {
    cpuctl: Cpuctl,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            cpuctl: Cpuctl {
                bg_uclamp: Uclamp { max: 0, min: 0 },
                ta_uclamp: Uclamp { max: 0, min: 0 },
                fg_uclamp: Uclamp { max: 0, min: 0 },
            },
        }
    }
    pub fn set_uclamp(&self) {
        let operations = [
            (
                "/dev/cpuctl/background/cpu.uclamp.max",
                self.cpuctl.bg_uclamp.max,
            ),
            (
                "/dev/cpuctl/background/cpu.uclamp.min",
                self.cpuctl.bg_uclamp.min,
            ),
            (
                "/dev/cpuctl/foreground/cpu.uclamp.max",
                self.cpuctl.fg_uclamp.max,
            ),
            (
                "/dev/cpuctl/foreground/cpu.uclamp.min",
                self.cpuctl.fg_uclamp.min,
            ),
            (
                "/dev/cpuctl/top-app/cpu.uclamp.max",
                self.cpuctl.ta_uclamp.max,
            ),
            (
                "/dev/cpuctl/top-app/cpu.uclamp.min",
                self.cpuctl.ta_uclamp.min,
            ),
        ];
        for (path, value) in operations {
            if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o644)) {
                log::error!("无法设置权限 {}: {}", path, e);
            }
            if let Err(e) = fs::write(path, value.to_string()) {
                log::error!("无法写入文件 {}: {}", path, e);
            }
        }
    }
}

pub fn lock_value(path: &str, value: &str) -> Result<()> {
    let mount_path = format!("/cache/mount_mask_{value}");
    unmount(path)?;
    if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o644)) {
        log::error!("无法设置权限{}: {e}", path);
    }
    if let Err(e) = fs::write(path, value) {
        log::error!("无法写入文件{}: {e}", path);
    }
    if let Err(e) = fs::set_permissions(path, fs::Permissions::from_mode(0o444)) {
        log::error!("无法设置权限{}: {e}", path);
    }
    if let Err(e) = fs::write(&mount_path, value) {
        log::error!("无法写入文件{}: {e}", mount_path);
    }
    mount_bind(&mount_path, path)?;
    Ok(())
}

fn mount_bind(src_path: &str, dest_path: &str) -> Result<()> {
    let src_path = CString::new(src_path)?;
    let dest_path = CString::new(dest_path)?;

    unsafe {
        umount2(dest_path.as_ptr(), libc::MNT_DETACH);

        if mount(
            src_path.as_ptr().cast(),
            dest_path.as_ptr().cast(),
            ptr::null(),
            MS_BIND | MS_REC,
            ptr::null(),
        ) != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(())
}

fn unmount(file_system: &str) -> Result<()> {
    let path = CString::new(file_system)?;
    if unsafe { umount(path.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
