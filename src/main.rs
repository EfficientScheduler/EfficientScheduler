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

use std::{fs, process::exit};

use anyhow::Result;

mod framework;
mod logger;

fn wait_boot() {
    while android_system_properties::AndroidSystemProperties::new()
        .get("sys.boot_completed")
        .unwrap_or_default()
        .contains("1")
    {
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn check_process() {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let pid = entry.file_name().into_string().unwrap_or_default();
            if pid.parse::<u32>().is_err() {
                continue;
            }
            if let Ok(cmdline) = fs::read_to_string(format!("/proc/{pid}/cmdline")) {
                if cmdline.contains("EfficientScheduler") {
                    count += 1;
                }
            }
        }
    }
    if count > 1 {
        eprintln!("发现另一个进程，程序退出");
        exit(1);
    }
}

fn main() -> Result<()> {
    logger::log_init()?;
    wait_boot();
    check_process();
    framework::scheduler::Scheduler::try_start_scheduler()?;
    Ok(())
}
