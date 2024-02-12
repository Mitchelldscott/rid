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
use hidapi::HidDevice;

use chrono::{DateTime, Utc};
use std::{sync::mpsc::Sender, time::Instant};

use crate::{host::layer::*, RIDReport, RID_PACKET_SIZE};

/// Reads from an Hid Device and send the packets through a channel
pub struct HidReader {
    parser_tx: Sender<(RIDReport, DateTime<Utc>)>,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidReader {
    pub fn new(layer: HidLayer, parser_tx: Sender<(RIDReport, DateTime<Utc>)>) -> HidReader {
        HidReader {
            parser_tx: parser_tx,
            teensy: layer.wait_for_device(),
            layer: layer,
            timestamp: Instant::now(),
        }
    }

    pub fn reconnect(&mut self) {
        // check reconnect after 1000 cycles
        if self.timestamp.elapsed().as_millis() as f64 > self.layer.sample_time {
            if self.layer.control_flags.is_connected() {
                println!(
                    "[HID-Reader]: hasn't written for {}s",
                    (self.timestamp.elapsed().as_millis() as f64) * 1E-3
                );
            }

            self.teensy = self.layer.wait_for_device();
        }
    }

    /// Read data into the input buffer and return how many bytes were read
    ///
    /// # Usage
    ///
    /// ```
    /// let reader = HidReader::new();
    /// match reader.read() {
    ///     64 => {
    ///         // packet OK, do something
    ///     }
    ///     _ => {} // do nothing
    /// }
    /// ```
    pub fn read(&mut self) -> usize {
        let mut buffer = [0; RID_PACKET_SIZE];
        match &self.teensy.read(&mut buffer) {
            Ok(value) => {
                if *value == RID_PACKET_SIZE {
                    match self.parser_tx.send((buffer, Utc::now())) {
                        Ok(_) => {}
                        _ => self.layer.control_flags.shutdown(),
                    };

                    self.layer.pc_stats.update_rx(1.0);
                    self.timestamp = Instant::now();
                }

                *value
            }
            _ => {
                self.layer.control_flags.initialize(false);
                self.reconnect();
                0
            }
        }
    }

    pub fn read_raw(&mut self) -> (Option<RIDReport>, DateTime<Utc>) {

        let mut buffer = [0; RID_PACKET_SIZE];

        match &self.teensy.read(&mut buffer) {
            Ok(value) => {
                if *value == RID_PACKET_SIZE {
                    self.layer.pc_stats.update_rx(1.0);
                    self.timestamp = Instant::now();
                    return (Some(buffer), Utc::now());
                }

            }
            _ => {
                self.layer.control_flags.initialize(false);
                self.reconnect();
            }
        }

        (None, Utc::now())
    }

    /// Main function to spin and connect the teensys
    /// input to Socks.
    ///
    /// # Usage
    /// ```
    /// ```
    pub fn spin(&mut self) {
        while !self.layer.control_flags.is_shutdown() {
            let t = Instant::now();

            self.read();

            self.layer.delay(t);
            // if self.layer.delay(t) > 1000.0 {
            //     println!("[HID-Reader]: over cycled {:.6}s", 1E-6 * (t.elapsed().as_micros() as f64));
            // }
        }
    }

    pub fn pipeline(&mut self) {
        println!("[HID-reader]: Live");

        self.spin();

        println!("[HID-reader]: Shutdown");
    }
}
