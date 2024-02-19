// /********************************************************************************
//  *
//  *      ____                     ____          __           __       _
//  *     / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
//  *    / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
//  *   / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  )
//  *  /_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/
//  *        /____/
//  *
//  *
//  *
//  ********************************************************************************/

// use crate::{
//     host::{layer::*, reader::*, writer::*},
//     RIDReport,
// };

// use std::{
//     sync::mpsc::{channel, Receiver, Sender},
//     time::Instant,
// };

// use chrono::{DateTime, Utc};

// pub static MCU_NO_COMMS_TIMEOUT_S: u64 = 10;
// pub static MCU_NO_COMMS_RESET_MS: u128 = 10;
// pub static MCU_RECONNECT_DELAY_US: f64 = 5.0 * 1E6;

// pub static TEENSY_CYCLE_TIME_S: f64 = 0.001;
// pub static TEENSY_CYCLE_TIME_MS: f64 = TEENSY_CYCLE_TIME_S * 1E3;
// pub static TEENSY_CYCLE_TIME_US: f64 = TEENSY_CYCLE_TIME_S * 1E6;
// pub static TEENSY_CYCLE_TIME_ER: f64 = TEENSY_CYCLE_TIME_US + 50.0; // err threshold (before prints happen, deprecated?)

// pub static TEENSY_DEFAULT_VID: u16 = 0x1331;
// pub static TEENSY_DEFAULT_PID: u16 = 0x0001;

// pub struct HidInterface {
//     pub layer: HidLayer,

//     // For sending reports to the writer
//     // pub reader_rx: Receiver<(RIDReport, DateTime<Utc>)>,
//     // pub writer_tx: Sender<RIDReport>,
// }

// impl HidInterface {
//     pub fn new() -> HidInterface {
//         let layer = HidLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);
//         // let (writer_tx, writer_rx) = channel::<RIDReport>();
//         // let (reader_tx, reader_rx) = channel::<(RIDReport, DateTime<Utc>)>();

//         // (
//             HidInterface {
//                 layer: layer.clone(),
//                 // reader_rx: reader_rx,
//                 // writer_tx: writer_tx,
//             }
//             // HidReader::new(layer.clone(), reader_tx),
//             // HidWriter::new(layer, writer_rx),
//         // )
//     }

//     pub fn sim() -> HidInterface {
//         let hidui = HidInterface::new();
//         hidui
//     }

//     // pub fn writer_tx(&self, buffer: RIDReport) {
//     //     match self.writer_tx.send(buffer) {
//     //         Ok(_) => {}
//     //         _ => self.layer.control_flags.shutdown(),
//     //     };
//     // }

//     // pub fn check_feedback(&mut self) {
//     //     match self.reader_rx.try_recv() {
//     //         Ok((_buffer, _datetime)) => {

//     //             // let pc_time = self.layer.pc_stats.from_utcs(datetime, self.layer.datetime) as f64;
//     //         }
//     //         _ => {}
//     //     }
//     // }

//     // pub fn read_write_spin(&mut self, packets: Vec<RIDReport>) {
//     //     packets.into_iter().for_each(|packet| {
//     //         let t = Instant::now();
//     //         self.writer_tx(packet);
//     //         self.check_feedback();
//     //         self.layer.delay(t);
//     //     });
//     // }

//     // pub fn send_initializers(&mut self) {
//     //     self.read_write_spin(self.robot_fw.all_init_packets());
//     // }

//     // pub fn try_config(&mut self) {
//     //     self.read_write_spin(self.robot_fw.unconfigured_parameters());
//     // }

//     // pub fn pipeline(&mut self, _unused_flag: bool) {
//     //     while !self.layer.control_flags.is_connected() {}

//     //     let mut t = Instant::now();

//     //     println!("[HID-Control]: Live");

//     //     while !self.layer.control_flags.is_shutdown() {
//     //         let loopt = Instant::now();

//     //         if !self.layer.control_flags.is_connected()
//     //             || !self.layer.control_flags.is_initialized()
//     //         {
//     //             // self.robot_fw.configured = vec![false; self.robot_fw.tasks.len()];
//     //             self.layer.control_flags.initialize(true);
//     //             // self.send_initializers();
//     //         } else {
//     //             // self.try_config();

//     //             // match self.robot_fw.parse_sock() {
//     //             //     Some(packet) => self.writer_tx(packet),
//     //             //     _ => {}
//     //             // }

//     //             self.check_feedback();

//     //             if t.elapsed().as_secs() >= 20 {
//     //                 self.print();
//     //                 t = Instant::now();
//     //             }
//     //         }

//     //         // self.layer.delay(loopt);
//     //         if self.layer.delay(loopt) > TEENSY_CYCLE_TIME_US {
//     //             println!(
//     //                 "[HID-Control]: over cycled {:.6}s",
//     //                 1E-6 * (t.elapsed().as_micros() as f64)
//     //             );
//     //         }
//     //     }

//     //     // sockapi::shutdown();
//     //     self.layer.control_flags.shutdown();
//     //     println!("[HID-Control]: shutdown");
//     //     self.layer.print();
//     // }

//     // pub fn print(&self) {
//     //     self.layer.print();
//     //     // self.robot_fw.print();
//     // }
// }
