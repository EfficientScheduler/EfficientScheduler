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

use std::{
    collections::HashMap,
    fs::{self, write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::Mode;

pub struct Cpu {
    info: HashMap<usize, PathBuf>,
}

impl Cpu {
    pub fn new() -> Result<Self> {
        let sysfs = Path::new("/sys/devices/system/cpu/cpufreq/");
        let mut info = HashMap::new();
        for entry in fs::read_dir(sysfs)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(policy_name) = path.file_name().and_then(|n| n.to_str()) {
                if !policy_name.starts_with("policy") {
                    continue;
                }
                let cpu_id = policy_name[6..].parse::<usize>()?;
                info.insert(cpu_id, path);
            }
        }
        Ok(Self { info })
    }

    pub fn set_freqs(&self, mode: Mode) {
        for (_, path) in self.info.clone() {
            let freq_max_path = path.join("scaling_max_freq");
            let freq_min_path = path.join("scaling_mim_freq");
            let freqs = fs::read_to_string(
                path.join("scaling_available_frequencies")
                    .to_string_lossy()
                    .to_string()
                    .as_str(),
            )
            .unwrap();
            let context: Vec<isize> = freqs
                .split_whitespace()
                .filter_map(|s| s.parse::<isize>().ok())
                .collect();
            let max_freq: isize;
            let min_freq: isize;
            match mode {
                Mode::Powersave => {
                    max_freq = context[context.len() - 5];
                    min_freq = context[context.len() - 3];
                }
                Mode::Balance => {
                    max_freq = context[5];
                    min_freq = context[context.len() - 6];
                }
                Mode::Performance => {
                    max_freq = context[1];
                    min_freq = context[context.len() - 6];
                }
                Mode::Fast => {
                    max_freq = context[1];
                    min_freq = context[1];
                }
            }
            if let Err(e) = fs::set_permissions(&freq_max_path, fs::Permissions::from_mode(0o644)) {
                log::error!("无法设置权限{}: {e}", path.display());
            }
            if let Err(e) = fs::set_permissions(&freq_min_path, fs::Permissions::from_mode(0o644)) {
                log::error!("无法设置权限{}: {e}", path.display());
            }
            if let Err(e) = write(&path, max_freq.to_string().as_bytes()) {
                log::error!("无法写入频率{}: {e}", freq_max_path.display());
            }
            if let Err(e) = write(&path, min_freq.to_string().as_bytes()) {
                log::error!("无法写入频率{}: {e}", freq_min_path.display());
            }
            if let Err(e) = fs::set_permissions(&freq_max_path, fs::Permissions::from_mode(0o444)) {
                log::error!("无法设置权限{}: {e}", path.display());
            }
            if let Err(e) = fs::set_permissions(&freq_min_path, fs::Permissions::from_mode(0o444)) {
                log::error!("无法设置权限{}: {e}", path.display());
            }
            #[cfg(debug_assertions)]
            {
                log::debug!("已为{policy}设置频率");
            }
        }
    }
}
