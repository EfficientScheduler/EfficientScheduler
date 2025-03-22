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

#![allow(clippy::pedantic)]

use std::{fs, os::unix::fs::PermissionsExt};

use serde::Deserialize;

use super::Mode;

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
    mode: Mode,
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            mode: Mode::Balance,
            cpuctl: Cpuctl {
                bg_uclamp: Uclamp { max: 0, min: 0 },
                ta_uclamp: Uclamp { max: 0, min: 0 },
                fg_uclamp: Uclamp { max: 0, min: 0 },
            },
        }
    }

    pub const fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub const fn match_uclamp(&mut self) {
        match self.mode {
            Mode::Powersave => {
                self.cpuctl = Cpuctl {
                    bg_uclamp: Uclamp { max: 10, min: 5 },
                    ta_uclamp: Uclamp { max: 20, min: 10 },
                    fg_uclamp: Uclamp { max: 25, min: 20 },
                };
            }
            Mode::Balance => {
                self.cpuctl = Cpuctl {
                    bg_uclamp: Uclamp { max: 15, min: 10 },
                    ta_uclamp: Uclamp { max: 25, min: 20 },
                    fg_uclamp: Uclamp { max: 30, min: 20 },
                };
            }
            Mode::Performance => {
                self.cpuctl = Cpuctl {
                    bg_uclamp: Uclamp { max: 40, min: 10 },
                    ta_uclamp: Uclamp { max: 40, min: 20 },
                    fg_uclamp: Uclamp { max: 60, min: 20 },
                };
            }
            Mode::Fast => {
                self.cpuctl = Cpuctl {
                    bg_uclamp: Uclamp { max: 100, min: 10 },
                    ta_uclamp: Uclamp { max: 100, min: 20 },
                    fg_uclamp: Uclamp { max: 100, min: 20 },
                };
            }
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
