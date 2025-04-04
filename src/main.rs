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

#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]

use std::{fs, process::exit};

use anyhow::Result;
use sysinfo::{Pid, System};

mod framework;
mod logger;

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

fn kill_other_process() {
    let system = System::new_all();
    for i in system.processes_by_name("uperf".as_ref()) {
        let uperf = format!("{}", i.name().to_string_lossy());
        if !uperf.is_empty() {
            if let Some(process) = system.process(Pid::from_u32(i.pid().as_u32())) {
                process.kill();
            }
        }
    }
    for i in system.processes_by_name("fas-rs".as_ref()) {
        let uperf = format!("{}", i.name().to_string_lossy());
        if !uperf.is_empty() {
            if let Some(process) = system.process(Pid::from_u32(i.pid().as_u32())) {
                process.kill();
            }
        }
    }
    for i in system.processes_by_name("AsoulOpt".as_ref()) {
        let uperf = format!("{}", i.name().to_string_lossy());
        if !uperf.is_empty() {
            if let Some(process) = system.process(Pid::from_u32(i.pid().as_u32())) {
                process.kill();
            }
        }
    }
    for i in system.processes_by_name("AppOpt".as_ref()) {
        let uperf = format!("{}", i.name().to_string_lossy());
        if !uperf.is_empty() {
            if let Some(process) = system.process(Pid::from_u32(i.pid().as_u32())) {
                process.kill();
            }
        }
    }
}

fn main() -> Result<()> {
    logger::log_init()?;
    check_process();
    kill_other_process();
    let _ = fs::write(
        "/dev/cpuset/background/cgroup.procs",
        std::process::id().to_string(),
    );
    framework::scheduler::Scheduler::try_start_scheduler()?;
    Ok(())
}
