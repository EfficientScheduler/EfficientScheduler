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

use std::fs::read_to_string;

use anyhow::Result;

use super::ConfigData;

pub mod dump;
pub mod looper;

pub struct Scheduler;

impl Scheduler {
    pub fn try_start_scheduler() -> Result<()> {
        let context = read_to_string("/data/data/com.termux/files/home/.local/share/tmoe-linux/containers/chroot/arch_arm64/home/hutao/AstraPulse/modules/config.toml")?;
        let context: ConfigData = toml::from_str(context.as_str())?;
        looper::Looper::new(context).enter_looper();
        Ok(())
    }
}
