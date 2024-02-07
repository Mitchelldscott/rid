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

use crate::{host::layer::*, RIDReport, RID_PACKET_SIZE};

// use chrono::Utc;
use std::{sync::mpsc::Receiver, time::Instant};

pub struct HidWriter {
    writer_rx: Receiver<RIDReport>,
    teensy: HidDevice,
    layer: HidLayer,
    timestamp: Instant,
}

impl HidWriter {
    pub fn new(layer: HidLayer, writer_rx: Receiver<RIDReport>) -> HidWriter {
        HidWriter {
            writer_rx: writer_rx,
            teensy: layer.wait_for_device(),
            layer: layer,
            timestamp: Instant::now(),
        }
    }

    pub fn print(&self) {
        println!(
            "Writer Dump\n\ttimer: {} us\n\tpackets: {}",
            self.timestamp.elapsed().as_micros(),
            self.layer.pc_stats.n_tx(),
        );
    }

    pub fn silent_channel_default(&mut self) -> RIDReport {
        let mut buffer = [0; RID_PACKET_SIZE];
        buffer[0] = 255;
        buffer[1] = 255;
        (self.layer.pc_stats.n_tx() as f32)
            .to_be_bytes()
            .iter()
            .enumerate()
            .for_each(|(i, b)| buffer[i + 2] = *b);
        buffer
    }

    pub fn reconnect(&mut self) {
        // check reconnect after 1000 cycles
        if !self.layer.control_flags.is_shutdown()
            && self.timestamp.elapsed().as_millis() as f64 > self.layer.sample_time
        {
            if self.layer.control_flags.is_connected() {
                println!(
                    "[HID-Writer]: disconnecting, hasn't written for {}s",
                    (self.timestamp.elapsed().as_millis() as f64) * 1E-3
                );

                self.layer.control_flags.disconnect();
            }

            self.teensy = self.layer.wait_for_device();
        }
    }

    /// Write the bytes from the buffer to the teensy.
    /// Reconnect if the write fails.
    /// # Usage
    /// ```
    /// let mut buffer = [0; RID_PACKET_SIZE];
    /// let writer = HidWriter::new(layer, writer_rx);
    /// writer.write(buffer); // writes some_data to the teensy
    /// ```
    pub fn write(&mut self, buffer: &mut RIDReport) {
        // (1E-6 * (Utc::now().timestamp_micros() - self.layer.datetime.timestamp_micros()) as f32)
        //     .to_le_bytes()
        //     .iter()
        //     .enumerate()
        //     .for_each(|(i, &b)| buffer[HID_PCTS_INDEX + i] = b);

        // (self.layer.mcu_stats.time() as f32)
        //     .to_le_bytes()
        //     .iter()
        //     .enumerate()
        //     .for_each(|(i, &b)| buffer[HID_UCTS_INDEX + i] = b);

        match self.teensy.write(buffer) {
            Ok(value) => {
                if value == RID_PACKET_SIZE {
                    self.timestamp = Instant::now();
                    self.layer.pc_stats.update_tx(1.0);
                }
            }
            _ => {
                self.layer.control_flags.initialize(false);
                self.reconnect();
            }
        }
    }

    /// Continually sends data from 'writer_rx' to the teensy.
    ///
    ///
    /// # Example
    /// See [`HidLayer::pipeline()`] source
    pub fn pipeline(&mut self) {
        println!("[HID-writer]: Live");

        while !self.layer.control_flags.is_shutdown() {
            let t = Instant::now();

            let mut buffer = self
                .writer_rx
                .try_recv()
                .unwrap_or(self.silent_channel_default());

            self.write(&mut buffer);

            self.layer.delay(t);
            // if self.layer.delay(t) > 1000.0 {
            //     println!("[HID-writer]: over cycled {:.6}s", 1E-6 * (t.elapsed().as_micros() as f64));
            // }
        }

        let mut buffer = [13; RID_PACKET_SIZE];
        self.write(&mut buffer);

        println!("[HID-writer]: Shutdown");
    }
}
