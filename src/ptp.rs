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

use core::ops::Index;

pub const PTP_GAIN_INDEX: usize = 44;
pub const PTP_CRTS_INDEX: usize = 48;
pub const PTP_CWTS_INDEX: usize = 52;
pub const PTP_HRTS_INDEX: usize = 56;
pub const PTP_HWTS_INDEX: usize = 60;


pub const SEC_PER_HOUR: u64 = 3_600;
pub const USEC_PER_SEC: u32 = 1_000_000;
pub const SEC_PER_USEC: f32 = 1.0 / 1_000_000.0;
pub const USEC_PER_HOUR: u32 = USEC_PER_SEC * SEC_PER_HOUR as u32;

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

    pub fn offset(&self) -> f32 {
        ((self.host_write + self.host_read) as f32
                    - self.client_write as f32
                    - self.client_read as f32)
                    / 2.0
    }

    pub fn stamp(&mut self, buffer: &mut RIDReport) {
        buffer[PTP_CRTS_INDEX..PTP_CRTS_INDEX + 4].copy_from_slice(&self.client_read.to_be_bytes());
        buffer[PTP_CWTS_INDEX..PTP_CWTS_INDEX + 4]
            .copy_from_slice(&self.client_write.to_be_bytes());
        buffer[PTP_HRTS_INDEX..PTP_HRTS_INDEX + 4].copy_from_slice(&self.host_read.to_be_bytes());
        buffer[PTP_HWTS_INDEX..PTP_HWTS_INDEX + 4].copy_from_slice(&self.host_write.to_be_bytes());
    }

    pub fn host_read(&mut self, buffer: &RIDReport, timestamp: u32) {
        self.host_read = timestamp;

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

    pub fn client_read(&mut self, buffer: &RIDReport, timestamp: u32) {
        self.client_read = timestamp;

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

    pub fn host_stamp(&mut self, buffer: &mut RIDReport, timestamp: u32) {

        self.host_write = timestamp;
        self.stamp(buffer);
    
    }

    pub fn client_stamp(&mut self, buffer: &mut RIDReport, timestamp: u32) {

        self.client_write = timestamp;
        self.stamp(buffer);

    }
}

impl Index<usize> for TimeStamp {
    type Output = u32;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.client_read,
            1 => &self.client_write,
            2 => &self.host_read,
            _ => &self.host_write,
        }
    }
}

pub struct Duration {
    hours: u64,
    microseconds: u32, 
}

impl Duration {
    pub fn new(hours: u64, microseconds: u32) -> Duration {
        Duration {
            hours,
            microseconds,
        }
    }

    pub fn default() -> Duration {
        Duration::new(0, 0)
    }

    pub fn add_micros(&mut self, micros: u32) -> u32 {

        self.microseconds += micros as u32;

        while self.microseconds >= USEC_PER_HOUR {

            self.microseconds -= USEC_PER_HOUR;
            self.hours += 1;

            if self.hours == u64::MAX {
                self.hours = 0;
            }
        
        }

        self.microseconds
    }

    pub fn micros(&self) -> u32 {

        self.microseconds
    
    }

    pub fn millis(&mut self) -> u32 {

        self.microseconds / 1_000

    }

    pub fn time(&mut self) -> f32 {

        (self.hours * SEC_PER_HOUR) as f32 + (self.microseconds as f32 * SEC_PER_USEC)
    
    }
}