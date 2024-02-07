/********************************************************************************
 *
 *      ____                     ____          __           __       _
 *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
 *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
 *        /____/
 *
 *
 *
 ********************************************************************************/

extern crate hidapi;
use hidapi::{HidApi, HidDevice};

use chrono::{DateTime, Utc};
use std::time::Instant;

use crate::host::data_structures::{HidControlFlags, NetFlowStats};

pub struct HidLayer {
    // Device info for initializing connection
    pub vid: u16,
    pub pid: u16,
    pub sample_time: f64,

    pub hidapi: HidApi,

    pub datetime: DateTime<Utc>,

    // Layer Statistics
    pub pc_stats: NetFlowStats,
    pub mcu_stats: NetFlowStats,
    // Layer control vectors
    pub control_flags: HidControlFlags,
}

impl HidLayer {
    pub fn new(vid: u16, pid: u16, sample_time: f64) -> HidLayer {
        HidLayer {
            vid: vid,
            pid: pid,
            sample_time: sample_time,

            hidapi: HidApi::new().expect("Failed to create API instance"),

            datetime: Utc::now(),
            pc_stats: NetFlowStats::new(),
            mcu_stats: NetFlowStats::new(),
            control_flags: HidControlFlags::new(),
        }
    }

    pub fn clone(&self) -> HidLayer {
        HidLayer {
            vid: self.vid,
            pid: self.pid,
            sample_time: self.sample_time,

            hidapi: HidApi::new().expect("Failed to create API instance"),

            datetime: Utc::now(),
            pc_stats: self.pc_stats.clone(),
            mcu_stats: self.mcu_stats.clone(),
            control_flags: self.control_flags.clone(),
        }
    }

    pub fn device(&self) -> Option<HidDevice> {
        match self.hidapi.open(self.vid, self.pid) {
            Ok(dev) => {
                println!("New Device");
                self.control_flags.connect();
                dev.set_blocking_mode(false).unwrap();
                Some(dev)
            }
            Err(_) => {
                if self.control_flags.is_shutdown() {
                    panic!("[HID-Layer]: Shutdown while searching for MCU");
                }
                None
            }
        }
    }

    pub fn wait_for_device(&self) -> HidDevice {
        let mut lap_millis = 0;
        let mut lap_secs = 0;
        let t = Instant::now();

        while t.elapsed().as_secs() < 5 {
            match self.device() {
                Some(dev) => {
                    return dev;
                }
                None => {
                    let elapsed = t.elapsed();
                    if elapsed.as_secs() - lap_secs >= 1 {
                        println!(
                            "[HID-Layer]: Hasn't heard from MCU for {}s",
                            (elapsed.as_millis() as f64) * 1E-3
                        );
                        lap_secs = elapsed.as_secs();
                    }
                }
            }

            while ((t.elapsed().as_millis() - lap_millis) as f64) < 5.0 * self.sample_time {}
            lap_millis = t.elapsed().as_millis()
        }

        self.control_flags.shutdown();
        panic!("[HID-Layer]: Could not find MCU, shutting down");
    }

    pub fn delay(&self, time: Instant) -> f64 {
        let mut t = time.elapsed().as_micros() as f64;
        while t < self.sample_time {
            t = time.elapsed().as_micros() as f64;
        }
        t
    }

    pub fn print(&self) {
        println!("[HID-Layer]: {} {}", self.vid, self.pid);
        self.control_flags.print();
        println!("[PC]");
        self.pc_stats.print();
        println!("[MCU]");
        self.mcu_stats.print();
    }
}
