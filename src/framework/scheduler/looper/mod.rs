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

//mod buffer;
mod cpu;

use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use anyhow::Result;
use cpu::Cpu;
use frame_analyzer::Analyzer;

use crate::framework::ConfigData;

use super::dump::{power::Power, topapps::TopAppsWatcher};

#[derive(Clone)]
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
}

impl Looper {
    pub fn new(config: ConfigData) -> Self {
        Self {
            topapps: TopAppsWatcher::new(),
            power: Power::new(),
            config,
            cpu: Cpu::new().unwrap(),
            mode: Mode::Balance,
            last: Last { topapp: None },
        }
    }

    pub fn enter_looper(&mut self) {
        let _ = self.try_boost_run();
        loop {
            self.topapps.topapp_dumper();
            self.power.power_dumper();
            for (app, mode) in self.config.app.clone() {
                if self.last.topapp.clone().unwrap_or_default() != self.topapps.topapps
                    && self.topapps.topapps == app
                {
                    let _ = self.try_change_mode(mode.clone());
                    let _ = self.cpu.set_freqs(self.mode.clone());
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn try_change_mode(&mut self, mode: String) -> Result<()> {
        match mode.as_str() {
            "powersave" => self.mode = Mode::Powersave,
            "balance" => self.mode = Mode::Balance,
            "performance" => self.mode = Mode::Performance,
            "fast" => self.mode = Mode::Fast,
            _ => (),
        }
        Ok(())
    }

    fn try_boost_run(&mut self) -> Result<()> {
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
                let cmdline_path = format!("/proc/{}/cmdline", pid);
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
