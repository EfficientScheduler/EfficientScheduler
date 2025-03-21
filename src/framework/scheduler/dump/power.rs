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
    sync::LazyLock,
    time::{Duration, Instant},
};

use dumpsys_rs::Dumpsys;
use regex::Regex;

static WAKE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"mWakefulness=(Awake|Dreaming)").unwrap());
static SCREEN_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"mHoldingDisplaySuspendBlocker=(true)").unwrap());
static BRIGHTNESS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"mScreenBrightness=(\d+)").unwrap());
static LEGACY_SCREEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"mScreenOn=(true)").unwrap());

const RESET_TIME: Duration = Duration::from_secs(1);

pub struct Power {
    dumper: Dumpsys,
    pub state: bool,
    time: Instant,
}

impl Power {
    pub fn new() -> Self {
        let dumper = loop {
            if let Some(dump) = Dumpsys::new("power") {
                break dump;
            }
            log::error!("无法获取屏幕状态，正在重试");
            std::thread::sleep(Duration::from_secs(1));
        };
        Self {
            dumper,
            state: true,
            time: Instant::now(),
        }
    }

    pub fn power_dumper(&mut self) {
        if self.time.elapsed() > RESET_TIME {
            let dump = loop {
                match self.dumper.dump(&["state"]) {
                    Ok(dump) => break dump,
                    Err(e) => {
                        log::error!("无法获取屏幕状态：{e}，正在重试");
                        std::thread::sleep(Duration::from_secs(1));
                    }
                }
            };
            self.state = Self::parse_power(&dump);
            #[cfg(debug_assertions)]
            {
                log::debug!("当前屏幕状态 {}", self.state);
            }
        }
    }

    #[allow(clippy::pedantic)]
    fn parse_power(output: &str) -> bool {
        WAKE_REGEX.is_match(output)
            || SCREEN_BLOCK_REGEX.is_match(output)
            || LEGACY_SCREEN_REGEX.is_match(output)
            || BRIGHTNESS_REGEX
                .captures(output)
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .map(|bri| bri > 0)
                .unwrap_or(false)
    }
}
