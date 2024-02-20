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
//! # Precision Timing Protocol
//!
//!   This crate can be included in a firmware build (use the client calls) 
//! or built using the "std" feature.
//!

use crate::RIDReport;

use core::ops::Index;

/// Client read time index, client sets and host reads this
pub const PTP_CRTS_INDEX: usize = 48;
/// Client write time index, client sets and host reads this
pub const PTP_CWTS_INDEX: usize = 52;
/// Host read time index, host sets and client reads this
pub const PTP_HRTS_INDEX: usize = 56;
/// Client write time index, host sets and client reads this
pub const PTP_HWTS_INDEX: usize = 60;

/// Second to Hour constant: seconds = hours * SEC_PER_HOUR
pub const SEC_PER_HOUR: u64 = 3_600;
/// Microsecond to Second constant: microseconds = seconds * USEC_PER_SEC
pub const USEC_PER_SEC: u32 = 1_000_000;
/// Microsecond to Hour constant: microseconds = hours * USEC_PER_HOUR
pub const USEC_PER_HOUR: u32 = USEC_PER_SEC * SEC_PER_HOUR as u32;

#[derive(Debug)]
/// # Precision Timing Protocol implementation
/// This time stamp is used by a host and client to share
/// clock measurments based on message passing events. The algorithm
/// uses these events to estimate an offset between the host and
/// client system times. This offset can be used to syncronize messages
/// across machines. This will allow better data association and task 
/// scheduling.
pub struct TimeStamp {
    client_read: u32,
    client_write: u32,
    host_read: u32,
    host_write: u32,
}

impl TimeStamp {
    /// Create and return a [TimeStamp] with the given event times
    pub fn new(client_read: u32, client_write: u32, host_read: u32, host_write: u32) -> TimeStamp {
        TimeStamp {
            client_read: client_read,
            client_write: client_write,
            host_read: host_read,
            host_write: host_write,
        }
    }

    /// Calculates the PTP offset
    /// 
    /// most accurate when calculated on host read event.
    /// 
    /// Assumes events go: host write (t1) -> client read (t2) -> client write (t3) -> host read (t4)
    /// 
    /// C(t) = H(t) + o(t)
    ///
    /// This equation can be rearranged to calculate more accurately at different events
    /// 
    /// o(t) + d = (t2 - t1), -o(t) + d = (t4 - t3)
    /// 
    /// 2 * o = (t2 - t1) - (t4 + t3) - d + d
    /// 
    /// o(t) = (t2 + t3 - t1 - t4) / 2
    /// 
    pub fn offset(&self) -> f32 {
        (self.client_read as f32 
            + self.client_write as f32
            - self.host_read as f32
            - self.host_write as f32
        ) / 2.0
    }

    /// Write the microsecond event times to a report
    pub fn stamp(&mut self, buffer: &mut RIDReport) {
        buffer[PTP_CRTS_INDEX..PTP_CRTS_INDEX + 4].copy_from_slice(&self.client_read.to_be_bytes());
        buffer[PTP_CWTS_INDEX..PTP_CWTS_INDEX + 4]
            .copy_from_slice(&self.client_write.to_be_bytes());
        buffer[PTP_HRTS_INDEX..PTP_HRTS_INDEX + 4].copy_from_slice(&self.host_read.to_be_bytes());
        buffer[PTP_HWTS_INDEX..PTP_HWTS_INDEX + 4].copy_from_slice(&self.host_write.to_be_bytes());
    }

    /// Read the client event stamps from a report
    pub fn read_client_stamp(&self, buffer: &RIDReport) -> (u32, u32) {
        (
            u32::from_be_bytes([
                buffer[PTP_CRTS_INDEX],
                buffer[PTP_CRTS_INDEX + 1],
                buffer[PTP_CRTS_INDEX + 2],
                buffer[PTP_CRTS_INDEX + 3],
            ]),
            u32::from_be_bytes([
                buffer[PTP_CWTS_INDEX],
                buffer[PTP_CWTS_INDEX + 1],
                buffer[PTP_CWTS_INDEX + 2],
                buffer[PTP_CWTS_INDEX + 3],
            ])
        )
    }

    /// Read the host event stamps from a report
    pub fn read_host_stamp(&self, buffer: &RIDReport) -> (u32, u32) {
        (
            u32::from_be_bytes([
                buffer[PTP_HRTS_INDEX],
                buffer[PTP_HRTS_INDEX + 1],
                buffer[PTP_HRTS_INDEX + 2],
                buffer[PTP_HRTS_INDEX + 3],
            ]),
            u32::from_be_bytes([
                buffer[PTP_HWTS_INDEX],
                buffer[PTP_HWTS_INDEX + 1],
                buffer[PTP_HWTS_INDEX + 2],
                buffer[PTP_HWTS_INDEX + 3],
            ])
        )
    }

    /// update the host read time and save the clients event stamps
    pub fn host_read(&mut self, buffer: &RIDReport, timestamp: u32) {
        self.host_read = timestamp;

        (self.client_read, self.client_write) = self.read_client_stamp(buffer);
    }

    /// update the client read time and save the hosts event stamps
    pub fn client_read(&mut self, buffer: &RIDReport, timestamp: u32) {
        self.client_read = timestamp;

        (self.host_read, self.host_write) = self.read_host_stamp(buffer);
    }

    /// update the host write time and stamp a buffer
    pub fn host_stamp(&mut self, buffer: &mut RIDReport, timestamp: u32) {

        self.host_write = timestamp;
        self.stamp(buffer);
    
    }

    /// update the client write time and stamp a buffer
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

/// Lightweight timer object that
/// can run 2^64 hours + 2^32 microseconds,
/// Thats like... a long time.
pub struct Duration {
    hours: u64,
    microseconds: u32, 
}

impl Duration {
    /// create a ['Duration'] with the given hours and microseconds
    pub fn new(hours: u64, microseconds: u32) -> Duration {
        Duration {
            hours,
            microseconds,
        }
    }

    /// create a ['Duration'] with 0 hours and 0 microseconds
    pub fn default() -> Duration {
        Duration::new(0, 0)
    }

    /// add microseconds to the timer and handle overflow
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

    /// read the microseconds field
    pub fn micros(&self) -> u32 {

        self.microseconds
    
    }

    /// read the microseconds field as milliseconds
    pub fn millis(&mut self) -> u32 {

        self.microseconds / 1_000

    }

    /// read the timer value in seconds as a floating point
    pub fn time(&mut self) -> f32 {

        (self.hours * SEC_PER_HOUR) as f32 + (self.microseconds as f32 / USEC_PER_SEC as f32)
    
    }
}