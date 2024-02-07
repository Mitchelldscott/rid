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
//!
//!
//!
//! # Building
//!
//!
//! # Examples
//!
//!
//! # Configuration
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]
// #![warn(missing_docs)]

#[cfg(feature = "std")]
pub mod host;

pub const RID_PACKET_SIZE: usize = 64;

pub const RID_MODE_INDEX: usize = 0; // 255 = init data, 1 = overwrite data, 13 = kill
pub const RID_TOGL_INDEX: usize = 1; // init data: (1 = init task, 2 = config task) overwrite data: (latch)
pub const RID_TASK_INDEX: usize = 2; // only applies to init/overwrite data
pub const RID_DATA_INDEX: usize = 3; // data start

pub const PTP_GAIN_INDEX: usize = 44;
pub const PTP_CRTS_INDEX: usize = 48;
pub const PTP_CWTS_INDEX: usize = 52;
pub const PTP_HRTS_INDEX: usize = 56;
pub const PTP_HWTS_INDEX: usize = 60;

pub type RIDReport = [u8; RID_PACKET_SIZE];

pub struct Duration {
    hours: u64,
    seconds: u16,
    microseconds: u32, 
}

impl Duration {
    pub fn default() -> Duration {
        Duration {
            hours: 0,
            seconds: 0,
            microseconds: 0,
        }
    }

    pub fn add_micros(&mut self, micros: i32) -> u32 {
        
        if micros.is_negative() {

            let umicros = micros.abs() as u32;

            if umicros > self.microseconds {
                self.microseconds = u32::MAX - umicros;
                
                if self.seconds > 0 {
                    self.seconds -= 1;
                }
            }

            else {
                self.microseconds -= umicros;
            }

        }

        else {
            self.microseconds += micros as u32;
        }

        if self.microseconds >= 1_000_000 {

            self.microseconds -= 1_000_000;
            self.seconds += 1;
        
            if self.seconds >= 3600 {
                self.seconds -= 3600;
                self.hours += 1;
            }
        
        }

        self.millis()
    }

    pub fn millis(&mut self) -> u32 {
        // Always fits in u32
        // self.micros < 1_000_000 us = 1_000 ms
        // self.seconds < 3600 s = 3_600_000 ms
        // largest possible = 3_601_000 ms
        // every hour from start up this value resets
        (self.seconds as u32 * 1_000) + (self.microseconds / 1_000)
    }

    pub fn from_millis(&mut self, millis: u32) {
        self.seconds = (millis / 1_000) as u16;
        self.microseconds = millis % 1_000;
    }
}

#[derive(Debug)]
pub struct PTPStamp {
    gain: f32,
    prev_offset: i32,

    client_read: u32,
    client_write: u32,
    host_read: u32,
    host_write: u32,
}

impl PTPStamp {
    pub fn new(client_read: u32, client_write: u32, host_read: u32, host_write: u32) -> PTPStamp {
        PTPStamp {
            gain: 0.0,
            prev_offset: 0,
            client_read: client_read,
            client_write: client_write,
            host_read: host_read,
            host_write: host_write,
        }
    }

    pub fn from_report(buffer: &mut RIDReport) -> PTPStamp {
        let client_read = u32::from_be_bytes([
            buffer[PTP_CRTS_INDEX],
            buffer[PTP_CRTS_INDEX + 1],
            buffer[PTP_CRTS_INDEX + 2],
            buffer[PTP_CRTS_INDEX + 3],
        ]);

        let client_write = u32::from_be_bytes([
            buffer[PTP_CWTS_INDEX],
            buffer[PTP_CWTS_INDEX + 1],
            buffer[PTP_CWTS_INDEX + 2],
            buffer[PTP_CWTS_INDEX + 3],
        ]);

        let host_read = u32::from_be_bytes([
            buffer[PTP_HRTS_INDEX],
            buffer[PTP_HRTS_INDEX + 1],
            buffer[PTP_HRTS_INDEX + 2],
            buffer[PTP_HRTS_INDEX + 3],
        ]);

        let host_write = u32::from_be_bytes([
            buffer[PTP_HWTS_INDEX],
            buffer[PTP_HWTS_INDEX + 1],
            buffer[PTP_HWTS_INDEX + 2],
            buffer[PTP_HWTS_INDEX + 3],
        ]);

        PTPStamp {
            gain: 0.0,
            prev_offset: 0,
            client_read: client_read,
            client_write: client_write,
            host_read: host_read,
            host_write: host_write,
        }
    }

    pub fn marks(&self) -> (u32, u32, u32, u32) {
        (
            self.client_read,
            self.client_write,
            self.host_read,
            self.host_write,
        )
    }

    pub fn get_gain(&self) -> f32 {
        self.gain
    }

    pub fn offset(&self) -> i32 {
        ((self.host_write + self.host_read) as i32
            - self.client_write as i32
            - self.client_read as i32)
            / 2
    }

    pub fn adaptive_gain(&mut self) -> i32 {
        let offset = self.offset();
        let delta = 0.1 * (offset - self.prev_offset).signum() as f32;


        self.gain = self. gain + delta;
        self.prev_offset = offset;

        self.gain as i32
    }

    pub fn host_read(&mut self, buffer: &RIDReport, millis: u32) {
        self.host_read = millis;

        self.gain = f32::from_be_bytes([
            buffer[PTP_GAIN_INDEX],
            buffer[PTP_GAIN_INDEX + 1],
            buffer[PTP_GAIN_INDEX + 2],
            buffer[PTP_GAIN_INDEX + 3],
        ]);

        self.client_read = u32::from_be_bytes([
            buffer[PTP_CRTS_INDEX],
            buffer[PTP_CRTS_INDEX + 1],
            buffer[PTP_CRTS_INDEX + 2],
            buffer[PTP_CRTS_INDEX + 3],
        ]);

        self.client_write = u32::from_be_bytes([
            buffer[PTP_CWTS_INDEX],
            buffer[PTP_CWTS_INDEX + 1],
            buffer[PTP_CWTS_INDEX + 2],
            buffer[PTP_CWTS_INDEX + 3],
        ]);
    }

    pub fn host_stamp(&mut self, buffer: &mut RIDReport, millis: u32) {
        self.host_write = millis;

        buffer[PTP_CRTS_INDEX..PTP_CRTS_INDEX + 4].copy_from_slice(&self.client_read.to_be_bytes());
        buffer[PTP_CWTS_INDEX..PTP_CWTS_INDEX + 4]
            .copy_from_slice(&self.client_write.to_be_bytes());
        buffer[PTP_HRTS_INDEX..PTP_HRTS_INDEX + 4].copy_from_slice(&self.host_read.to_be_bytes());
        buffer[PTP_HWTS_INDEX..PTP_HWTS_INDEX + 4].copy_from_slice(&self.host_write.to_be_bytes());
    }

    pub fn host_read_elapsed(&self, millis: u32) -> u32 {
        millis - self.host_read
    }

    pub fn host_write_elapsed(&self, millis: u32) -> u32 {
        millis - self.host_write
    }

    pub fn client_read(&mut self, buffer: &RIDReport, millis: u32) -> i32 {
        self.client_read = millis;

        self.host_read = u32::from_be_bytes([
            buffer[PTP_HRTS_INDEX],
            buffer[PTP_HRTS_INDEX + 1],
            buffer[PTP_HRTS_INDEX + 2],
            buffer[PTP_HRTS_INDEX + 3],
        ]);

        self.host_write = u32::from_be_bytes([
            buffer[PTP_HWTS_INDEX],
            buffer[PTP_HWTS_INDEX + 1],
            buffer[PTP_HWTS_INDEX + 2],
            buffer[PTP_HWTS_INDEX + 3],
        ]);

        self.adaptive_gain()
    }

    pub fn client_stamp(&mut self, buffer: &mut RIDReport, millis: u32) {
        self.client_write = millis;

        buffer[PTP_GAIN_INDEX..PTP_GAIN_INDEX + 4].copy_from_slice(&self.gain.to_be_bytes());
        buffer[PTP_CRTS_INDEX..PTP_CRTS_INDEX + 4].copy_from_slice(&self.client_read.to_be_bytes());
        buffer[PTP_CWTS_INDEX..PTP_CWTS_INDEX + 4]
            .copy_from_slice(&self.client_write.to_be_bytes());
        buffer[PTP_HRTS_INDEX..PTP_HRTS_INDEX + 4].copy_from_slice(&self.host_read.to_be_bytes());
        buffer[PTP_HWTS_INDEX..PTP_HWTS_INDEX + 4].copy_from_slice(&self.host_write.to_be_bytes());
    }

    pub fn client_read_elapsed(&self, millis: u32) -> u32 {
        millis - self.client_read
    }

    pub fn client_write_elapsed(&self, millis: u32) -> u32 {
        millis - self.client_write
    }
}


pub struct RTTask {

}