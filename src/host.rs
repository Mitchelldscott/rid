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

use crate::{
    RIDReport, RID_PACKET_SIZE, RID_CYCLE_TIME_US,
    ptp::{Duration, TimeStamp}
};

/// helper function to create a new HidDevice
/// not really relevant since monothread
pub fn new_device(vid: u16, pid: u16, hidapi: &mut HidApi) -> HidDevice {
        
    let device = hidapi.open(vid, pid).expect("[HID-Layer] Failed to open device");
            
    device.set_blocking_mode(false).unwrap();

    device
}


/// the host side interface to the microcontroller
/// this should provide an abstraction for the
/// task deploy system to utilize.
pub struct RIDLayer {
    /// USB device vid
    pub vid: u16,
    /// USB device pid
    pub pid: u16,

    /// USB hidapi (C wrapper lib)
    pub hidapi: HidApi,
    /// USB HidDevice class
    pub device: HidDevice,

    /// ['Duration'] keeps track of host "system_time", H(t)
    pub system_time: Duration,
    /// ['TimeStamp'] for synchronization
    pub ptp_stamp: TimeStamp,

}

impl RIDLayer {
    /// Create a new RID layer
    /// Connects to the device with the specified vid, pid
    /// panics if the device cant be found. If it does not 
    /// have permission check your udev rules and make sure
    /// it includes the vid pid.
    pub fn new(vid: u16, pid: u16) -> RIDLayer {

        let mut hidapi = HidApi::new().expect("Failed to create API instance");
        let device = new_device(vid, pid, &mut hidapi);

        let system_time = Duration::default();
        let ptp_stamp = TimeStamp::new(0, 0, 0, 0);

        RIDLayer {
            vid,
            pid,

            hidapi,
            device,

            system_time,
            ptp_stamp,

        }
    }

    /// try reading a Report into a buffer
    pub fn read(&mut self, buffer: &mut RIDReport, micros: u32) -> usize {
        
        match self.device.read(buffer) {
            Ok(val) => {

                self.ptp_stamp.host_read(buffer, self.system_time.micros() + micros);
                
                val

            },
            _ => {

                println!("[HID-Layer] Failed to read");
                
                0
            
            },
        }

    }

    /// try writing a Report from a buffer
    pub fn write(&mut self, buffer: &mut RIDReport, micros: u32) {
        
        self.ptp_stamp.host_stamp(buffer, self.system_time.micros() + micros);

        match self.device.write(buffer) {
            Ok(RID_PACKET_SIZE) => {},
            _ => println!("[HID-Layer] Failed to write"),
        }

    }

    /// Delay helper, makes loops readable
    pub fn delay(&self, time: Instant) -> u32 {
        let mut t = time.elapsed().as_micros();

        while t < RID_CYCLE_TIME_US as u128 {
            t = time.elapsed().as_micros();
        }
        
        t as u32
    }

    /// another delay helper, makes loops real nice
    pub fn timestep(&mut self, t: Instant) -> u32 {
        self.system_time.add_micros(self.delay(t))
    }
}


