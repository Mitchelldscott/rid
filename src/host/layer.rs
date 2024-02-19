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

// use chrono::{DateTime, Utc};
use std::time::Instant;

use crate::{RIDReport, RID_PACKET_SIZE, ptp::{Duration, TimeStamp}};
// use crate::host::data_structures::{HidControlFlags, NetFlowStats};

pub static MCU_NO_COMMS_TIMEOUT_S: u64 = 10;
pub static MCU_NO_COMMS_RESET_MS: u128 = 10;
pub static MCU_RECONNECT_DELAY_US: f64 = 5.0 * 1E6;

pub static TEENSY_CYCLE_TIME_S: f64 = 0.001;
pub static TEENSY_CYCLE_TIME_MS: f64 = TEENSY_CYCLE_TIME_S * 1E3;
pub static TEENSY_CYCLE_TIME_US: f64 = TEENSY_CYCLE_TIME_S * 1E6;
pub static TEENSY_CYCLE_TIME_ER: f64 = TEENSY_CYCLE_TIME_US + 50.0; // err threshold (before prints happen, deprecated?)

pub static TEENSY_DEFAULT_VID: u16 = 0x1331;
pub static TEENSY_DEFAULT_PID: u16 = 0x0001;

pub struct RIDLayer {
    // Device info for initializing connection
    pub vid: u16,
    pub pid: u16,
    pub sample_time: f64,

    pub hidapi: HidApi,
    pub device: HidDevice,

    pub system_time: Duration,
    pub ptp_stamp: TimeStamp,
    pub connected: bool,


    // Layer Statistics
    // pub pc_stats: NetFlowStats,
    // pub mcu_stats: NetFlowStats,
    // Layer control vectors
    // pub control_flags: HidControlFlags,
}

pub fn new_device(vid: u16, pid: u16, hidapi: &mut HidApi) -> HidDevice {
        
        let device = hidapi.open(vid, pid).expect("[HID-Layer] Failed to open device");
                
        device.set_blocking_mode(false).unwrap();

        device
    }

impl RIDLayer {
    pub fn new(vid: u16, pid: u16, sample_time: f64) -> RIDLayer {

        let mut hidapi = HidApi::new().expect("Failed to create API instance");
        let device = new_device(vid, pid, &mut hidapi);

        let system_time = Duration::default();
        let ptp_stamp = TimeStamp::new(0, 0, 0, 0);
        let connected = false;

        RIDLayer {
            vid,
            pid,
            sample_time,

            hidapi,
            device,

            system_time,
            ptp_stamp,
            connected,

        }
    }

    pub fn read(&mut self, buffer: &mut RIDReport) -> usize {
        
        match self.device.read(buffer) {
            Ok(val) => {

                self.connected = true;
                self.ptp_stamp.host_read(buffer, self.system_time.micros());
                
                val

            },
            _ => {

                println!("[HID-Layer] Failed to read");
                
                0
            
            },
        }

    }

    pub fn write(&mut self, buffer: &mut RIDReport) {
        
        self.ptp_stamp.host_stamp(buffer, self.system_time.micros());

        match self.device.write(buffer) {
            Ok(RID_PACKET_SIZE) => {},
            _ => println!("[HID-Layer] Failed to write"),
        }

    }

    pub fn delay(&self, time: Instant) -> f64 {
        let mut t = time.elapsed().as_micros() as f64;

        while t < self.sample_time {
            t = time.elapsed().as_micros() as f64;
        }
        
        t
    }

    // pub fn print(&self) {
    //     println!("[HID-Layer]: {} {}", self.vid, self.pid);
    //     self.control_flags.print();
    //     println!("[PC]");
    //     self.pc_stats.print();
    //     println!("[MCU]");
    //     self.mcu_stats.print();
    // }
}


