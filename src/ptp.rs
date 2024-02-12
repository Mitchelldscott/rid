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

use crate::RIDReport;

pub const PTP_GAIN_INDEX: usize = 44;
pub const PTP_CRTS_INDEX: usize = 48;
pub const PTP_CWTS_INDEX: usize = 52;
pub const PTP_HRTS_INDEX: usize = 56;
pub const PTP_HWTS_INDEX: usize = 60;

#[derive(Debug)]
pub struct TimeStamp {
    client_read: u32,
    client_write: u32,
    host_read: u32,
    host_write: u32,
}

impl TimeStamp {
    pub fn new(client_read: u32, client_write: u32, host_read: u32, host_write: u32) -> TimeStamp {
        TimeStamp {
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

    pub fn offset(&self) -> i32 {
        ((self.host_write + self.host_read) as i32
                    - self.client_write as i32
                    - self.client_read as i32)
                    / 2
    }

    pub fn stamp(&mut self, buffer: &mut RIDReport) {
        buffer[PTP_CRTS_INDEX..PTP_CRTS_INDEX + 4].copy_from_slice(&self.client_read.to_be_bytes());
        buffer[PTP_CWTS_INDEX..PTP_CWTS_INDEX + 4]
            .copy_from_slice(&self.client_write.to_be_bytes());
        buffer[PTP_HRTS_INDEX..PTP_HRTS_INDEX + 4].copy_from_slice(&self.host_read.to_be_bytes());
        buffer[PTP_HWTS_INDEX..PTP_HWTS_INDEX + 4].copy_from_slice(&self.host_write.to_be_bytes());
    }

    pub fn host_read(&mut self, buffer: &RIDReport, millis: u32) {
        self.host_read = millis;

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

    pub fn client_read(&mut self, buffer: &RIDReport, millis: u32) {
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
    }

    pub fn host_stamp(&mut self, buffer: &mut RIDReport, millis: u32) {

        self.host_write = millis;
        self.stamp(buffer);
    
    }

    pub fn client_stamp(&mut self, buffer: &mut RIDReport, millis: u32) {

        self.client_write = millis;
        self.stamp(buffer);

    }
}

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

    pub fn time(&mut self) -> f32 {
        // Always fits in u32
        // self.micros < 1_000_000 us = 1_000 ms
        // self.seconds < 3600 s = 3_600_000 ms
        // largest possible = 3_601_000 ms
        // every hour from start up this value resets
        (self.hours as f32 * 3600.0) + (self.seconds as f32) + (self.microseconds as f32 / 1_000_000.0)
    }

    pub fn from_millis(&mut self, millis: u32) {
        self.seconds = (millis / 1_000) as u16;
        self.microseconds = millis % 1_000;
    }
}