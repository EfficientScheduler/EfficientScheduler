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

fn main() -> Result<()> {
    logger::log_init()?;
    framework::scheduler::Scheduler::try_start_scheduler()?;
    Ok(())
}
