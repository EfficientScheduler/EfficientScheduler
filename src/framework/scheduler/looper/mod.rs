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

mod buffer;
mod cpu;

use std::{
    collections::VecDeque,
    ffi::CString,
    fs,
    os::unix::fs::PermissionsExt,
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use anyhow::Result;
use buffer::Buffer;
use cpu::Cpu;
use frame_analyzer::Analyzer;
use libc::{MS_BIND, MS_REC, mount, umount, umount2};

use crate::framework::ConfigData;

use super::dump::{power::Power, topapps::TopAppsWatcher};

#[derive(Clone, Copy)]
pub enum Mode {
    Powersave,
    Balance,
    Performance,
    Fast,
}

struct Last {
    topapp: Option<String>,
}

pub struct Looper {
    topapps: TopAppsWatcher,
    power: Power,
    config: ConfigData,
    last: Last,
    cpu: Cpu,
    mode: Mode,
    buffer: Buffer,
}

impl Looper {
    pub fn new(config: ConfigData) -> Self {
        Self {
            topapps: TopAppsWatcher::new(),
            power: Power::new(),
            config,
            cpu: Cpu::new().unwrap(),
            mode: Mode::Balance,
            buffer: Buffer::new(),
            last: Last { topapp: None },
        }
    }

    fn disable() -> Result<()> {
        lock_value("/sys/module/mtk_fpsgo/parameters/perfmgr_enable", "0")?;
        lock_value("/sys/module/perfmgr/parameters/perfmgr_enable", "0")?;
        lock_value("/sys/module/perfmgr_policy/parameters/perfmgr_enable", "0")?;
        lock_value("/sys/module/perfmgr_mtk/parameters/perfmgr_enable", "0")?;
        lock_value("/sys/module/migt/parameters/glk_fbreak_enable", "0")?;
        lock_value("/sys/module/migt/parameters/glk_disable", "1")?;
        lock_value("/proc/game_opt/disable_cpufreq_limit", "1")?;
        Ok(())
    }

    pub fn enter_looper(&mut self) {
        let _ = Self::disable();
        #[cfg(debug_assertions)]
        {
            log::debug!("已关闭大部分系统自带功能");
        }
        let _ = self.try_boost_run();
        loop {
            self.topapps.topapp_dumper();
            self.power.power_dumper();
            if self.power.state {
                for (app, mode) in self.config.app.clone() {
                    if self.last.topapp.clone().unwrap_or_default() != self.topapps.topapps
                        && self.topapps.topapps == app
                    {
                        match mode.as_str() {
                            "powersave" => self.mode = Mode::Powersave,
                            "balance" => self.mode = Mode::Balance,
                            "performance" => self.mode = Mode::Performance,
                            "fast" => self.mode = Mode::Fast,
                            _ => log::error!("无效的Mode"),
                        }
                        self.last.topapp = Some(self.topapps.topapps.clone());
                    } else {
                        match self.config.on.as_str() {
                            "powersave" => self.mode = Mode::Powersave,
                            "balance" => self.mode = Mode::Balance,
                            "performance" => self.mode = Mode::Performance,
                            "fast" => self.mode = Mode::Fast,
                            _ => log::error!("无效的Mode"),
                        }
                    }
                }
            } else {
                match self.config.off.as_str() {
                    "powersave" => self.mode = Mode::Powersave,
                    "balance" => self.mode = Mode::Balance,
                    "performance" => self.mode = Mode::Performance,
                    "fast" => self.mode = Mode::Fast,
                    _ => log::error!("无效的Mode"),
                }
            }
            let () = self.cpu.set_freqs(self.mode);
            self.buffer.set_mode(self.mode);
            self.buffer.match_uclamp();
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn try_boost_run(&self) -> Result<()> {
        let mut analyzer = Analyzer::new()?;
        analyzer.attach_app(Self::find_pid(self.topapps.topapps.as_str())? as i32)?;
        let running = Arc::new(AtomicBool::new(true));
        let mut buffer = VecDeque::with_capacity(120);
        thread::spawn(move || {
            let cpu = Cpu::new().unwrap();
            while running.load(Ordering::Acquire) {
                if let Some((_, frametime)) = analyzer.recv() {
                    if buffer.len() >= 120 {
                        buffer.pop_back();
                        buffer.push_front(frametime);
                    }
                    if buffer.len() <= 10 {
                        cpu.set_freqs(Mode::Fast);
                    }
                }
            }
        });
        Ok(())
    }

    fn find_pid(package_name: &str) -> Result<u32> {
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let pid_str = entry.file_name().into_string().ok().unwrap_or_default();
                let pid = pid_str.parse::<u32>()?;
                let cmdline_path = format!("/proc/{pid}/cmdline");
                if let Ok(cmdline) = std::fs::read_to_string(cmdline_path) {
                    if cmdline.trim_matches('\0').contains(package_name) {
                        return Ok(pid);
                    }
                }
            }
        }
        Ok(0)
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
