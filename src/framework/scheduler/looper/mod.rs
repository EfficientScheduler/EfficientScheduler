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

use anyhow::Result;

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
    mode: Mode,
}

impl Looper {
    pub fn new(config: ConfigData) -> Self {
        Self {
            topapps: TopAppsWatcher::new(),
            power: Power::new(),
            config,
            mode: Mode::Balance,
            last: Last { topapp: None },
        }
    }

    pub fn enter_looper(&mut self) {
        loop {
            self.topapps.topapp_dumper();
            self.power.power_dumper();
            for (app, mode) in self.config.app.clone() {
                if self.last.topapp.clone().unwrap_or_default() != self.topapps.topapps
                    && self.topapps.topapps == app
                {
                    let _ = self.try_change_mode(mode);
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
}
