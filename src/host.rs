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
    RIDReport, 
    RID_PACKET_SIZE, RID_CYCLE_TIME_US,
    RID_MODE_INDEX, RID_TOGL_INDEX,
    ptp::{Duration, TimeStamp, USEC_PER_SEC, SEC_PER_HOUR}
};

/// Microsecond to Hour constant: microseconds = hours * USEC_PER_HOUR
pub const USEC_PER_HOUR: f32 = USEC_PER_SEC as f32 * SEC_PER_HOUR as f32;

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

    // The number of hours that have elapsed on the host
    pub host_hours: f32,
    // The time the host started at
    pub host_start: f32,
    // The number of hours that have elapsed on the client
    pub client_hours: f32,
    // The time the client started at
    pub client_start: f32,

    // The linear offset coefficients
    pub linear_offset: [f32; 2],

    // [Instant] to track change in time
    pub timer: Instant,

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

        let system_time = Duration::new(USEC_PER_HOUR as u32 - 5_000_000);
        let ptp_stamp = TimeStamp::new(0, 0, 0, 0);

        let timer = Instant::now();

        let host_hours = 0.0;
        let host_start = system_time.micros() as f32;

        let client_hours = 0.0;
        let client_start = 0.0;

        let linear_offset = [0.0, 0.0];


        RIDLayer {
            vid,
            pid,

            host_hours,
            host_start,

            client_hours,
            client_start,

            linear_offset,

            timer,

            hidapi,
            device,

            system_time,
            ptp_stamp,

        }
    }

    /// try reading a Report into a buffer
    pub fn read(&mut self, buffer: &mut RIDReport) -> usize {
        
        match self.device.read(buffer) {
            Ok(val) => {

                self.ptp_stamp.host_read(buffer, self.system_time.micros() + self.timer.elapsed().as_micros() as u32);

                val

            },
            _ => {

                println!("[HID-Layer] Failed to read");
                
                0
            
            },
        }

    }

    /// try writing a Report from a buffer
    pub fn write(&mut self, buffer: &mut RIDReport) {
        
        self.ptp_stamp.host_stamp(buffer, self.system_time.micros() + self.timer.elapsed().as_micros() as u32);

        match self.device.write(buffer) {
            Ok(RID_PACKET_SIZE) => {},
            _ => println!("[HID-Layer] Failed to write"),
        }

    }

    pub fn ptp_offset(&self) -> f32 {

        self.ptp_stamp.offset() + ((self.client_hours - self.host_hours) * USEC_PER_HOUR)
    
    }

    pub fn host_elapsed(&self) -> f32 {

        (self.system_time.micros() + self.timer.elapsed().as_micros() as u32) as f32 
            + (self.host_hours * USEC_PER_HOUR) 
            - self.host_start

    }

    pub fn client_elapsed(&self) -> f32 {

        self.ptp_stamp[1] as f32 + (self.client_hours * USEC_PER_HOUR) - self.client_start

    }

    pub fn ptp_to_client(&self, t: f32) -> f32 {

        t + self.ptp_offset() 

    }

    pub fn linear_to_client(&self, t: f32) -> f32 {
        

        (self.linear_offset[0] * (t + self.host_start)) + self.linear_offset[1] - self.client_start

    }

    pub fn ptp_to_host(&self, t: f32) -> f32 {

        t - self.ptp_offset()

    }

    pub fn linear_to_host(&self, t: f32) -> f32 {
        
        ((t - self.linear_offset[1]) / self.linear_offset[0])

    }

    /// Delay helper, makes loops readable
    pub fn delay(&mut self) -> u32 {

        let mut t = self.timer.elapsed().as_micros();

        while t < RID_CYCLE_TIME_US as u128 {

            t = self.timer.elapsed().as_micros();
        
        }

        self.timer = Instant::now();
        
        t as u32

    }

    /// another delay helper, makes loops real nice
    pub fn timestep(&mut self) -> u32 {

        let t = self.delay();


        if self.system_time.micros() + t > USEC_PER_HOUR as u32 {

            self.host_hours += 1.0;
        
        }

        self.system_time.add_micros(t)

    }


    /// Write, try to read and update the ptp stamp and system time
    pub fn spin(&mut self) -> f32 {

        let mut buffer = [0; RID_PACKET_SIZE];
        buffer[RID_MODE_INDEX] = 255;
        buffer[RID_TOGL_INDEX] = 255;

        self.write(&mut buffer);

        let prev_host_read = self.ptp_stamp[2];
        let prev_client_read = self.ptp_stamp[0];

        match self.read(&mut buffer) {

            RID_PACKET_SIZE => {


                if self.client_start == 0.0 {

                    self.client_start = self.ptp_stamp[1] as f32;

                    return 0.0;
                
                }
                else {

                    // Handles hour counts wrapping
                    if self.ptp_stamp[1] < prev_client_read {

                        self.client_hours += 1.0;
                    }

                    let (_, hw) = self.ptp_stamp.read_host_stamp(&buffer);

                    // Updates the linear offset coefficients
                    self.linear_offset[0] = self.client_elapsed() / self.host_elapsed();
                    self.linear_offset[1] = self.client_start - (self.linear_offset[0] * self.host_start);

                    return self.ptp_stamp[2] as f32 - hw as f32;

                }

            }
            _ => 0.0,
        }

    }

    pub fn print_header(&self) {
        println!("\n[PTP-DEMO]\tC(t) = {:.3} * H(t) + {:.3}", self.linear_offset[0], self.linear_offset[1]);
        println!("Host (s)\t\tClient (s)\t\tConversion Error <host, client> (us)");
    }

    pub fn print(&self) -> (f32, f32) {

        let host_time = self.system_time.micros() as f32 + (self.host_hours * USEC_PER_HOUR);
        let client_time = self.ptp_stamp[1] as f32;

        let host_err = host_time - self.ptp_to_host(client_time);
        let client_err = client_time - self.ptp_to_client(host_time);

        println!("  {:.4}\t\t{:.4}\t\t{:.0}\t{:.0}", 
            self.system_time.micros() as f32  / 1_000_000.0,
            client_time / 1_000_000.0,
            host_err,
            client_err,
        );

        (host_err, client_err)
    }
}


