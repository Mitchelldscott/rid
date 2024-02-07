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

#![allow(unused_imports)]
#![allow(unused_macros)]
extern crate hidapi;
use hidapi::{HidApi, HidDevice};

use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
    thread::{spawn, Builder},
    time::{Duration, Instant},
};

use more_asserts::assert_le;

use gnuplot::{Caption, Color, Figure};

use rid::host::{interface::*, reader::*, writer::*};
use rid::*;

#[allow(dead_code)]
const VERBOSITY: usize = 1;
pub static TEST_DURATION: u32 = 30;

pub mod ptp_performance {

    use super::*;

    pub fn sim_interface(interface: HidInterface) {
        while !interface.layer.control_flags.is_connected() {}

        println!("[HID-Control]: Live");

        let mut ptp_offset_collection = vec![];
        let mut ptp_stamp = PTPStamp::new(0, 0, 0, 0);

        let mut last_offset = 0.0;

        let mut client_reads = vec![];
        let mut client_writes = vec![];
        let mut host_reads = vec![];

        let mut t = Instant::now();
        let mut system_time = rid::Duration::default();

        while system_time.millis() / 1_000 < TEST_DURATION && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();

            if interface.layer.control_flags.is_connected() {
                
                let mut buffer = [0; RID_PACKET_SIZE];
                buffer[RID_MODE_INDEX] = 255;
                buffer[RID_TOGL_INDEX] = 255;

                ptp_stamp.host_stamp(&mut buffer, system_time.millis());

                interface.writer_tx(buffer);

                match interface.reader_rx.try_recv() {
                    Ok((buffer, _datetime)) => {

                        ptp_stamp.host_read(&buffer, system_time.millis());

                        let (cr, cw, hr, _) = ptp_stamp.marks();
                        client_reads.push(cr);
                        client_writes.push(cw);
                        host_reads.push(hr);

                        let offset = ptp_stamp.offset() as f64;

                        let millis = system_time.add_micros(t.elapsed().as_micros() as i32);
                        t = Instant::now();

                        println!(
                            "PTP offset: {} ms, offset delta: {}, gain: {}",
                            offset,
                            offset - last_offset,
                            ptp_stamp.get_gain(),
                        );

                        println!("Timers\tLocal: {}\tPTP(MCU): {} s", 
                            millis as f32 / 1_000.0, 
                            (cr + cw) as f32 / 2_000.0);

                        last_offset = offset;
                        ptp_offset_collection.push(offset);
                    }
                    _ => {}
                }
            }

            interface.layer.delay(loopt);
            // if interface.delay(t) > TEENSY_CYCLE_TIME_US {
            //     println!("HID Control over cycled {}", t.elapsed().as_micros());
            // }
        }

        interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        interface.print();

        let ptp_mean =
            ptp_offset_collection.iter().sum::<f64>() / ptp_offset_collection.len() as f64;
        let ptp_std = (ptp_offset_collection
            .iter()
            .map(|offset| (offset - ptp_mean) * (offset - ptp_mean))
            .sum::<f64>()
            / ptp_offset_collection.len() as f64)
            .sqrt();

        println!(
            "PTP Offset stats: \n\tn samples: {}\n\t(mean, std){ptp_mean} {ptp_std} ms",
            ptp_offset_collection.len()
        );

        let x = (1..ptp_offset_collection.len())
            .map(|x| x as f64)
            .collect::<Vec<f64>>();
        // let _sigma_p = ptp_offset_collection
        //     .iter()
        //     .map(|x| x + (2.0 * ptp_std))
        //     .collect::<Vec<f64>>();
        // let _sigma_n = ptp_offset_collection
        //     .iter()
        //     .map(|x| x - (2.0 * ptp_std))
        //     .collect::<Vec<f64>>();

        let offset_slope = (ptp_offset_collection[0] as i32
            ..*ptp_offset_collection.last().unwrap() as i32)
            .map(|x| x as f64)
            .collect::<Vec<f64>>();

        let mut fg = Figure::new();
        fg.axes2d()
            .lines(
                &x,
                &ptp_offset_collection,
                &[Caption("PTP Offset (ms)"), Color("black")],
            )
            .lines(
                &x,
                &vec![ptp_mean; ptp_offset_collection.len()],
                &[Caption("Average"), Color("green")],
            );

        let _ = fg.show();

        let x1 = (1..host_reads.len())
            .map(|x| x as u32)
            .collect::<Vec<u32>>();

        let mut fg1 = Figure::new();
        fg1.axes2d()
            .lines(
                &x1,
                &client_reads,
                &[Caption("Client Read"), Color("red")],
            )
            .lines(
                &x1,
                &host_reads,
                &[Caption("Host Read + offset"), Color("green")],
            );

        let _ = fg1.show();
        // assert_le!(
        //     (interface.layer.pc_stats.n_tx() - interface.layer.mcu_stats.n_tx()).abs(),
        //     (TEST_DURATION * 5) as f64,
        //     "PC and MCU sent different numbers of packets"
        // );
        // assert_le!(
        //     ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S) - interface.layer.mcu_stats.n_tx()).abs(),
        //     (TEST_DURATION * 500) as f64,
        //     "Not enough packts sent by mcu"
        // );
        // assert_le!(
        //     ((TEST_DURATION as f64 / TEENSY_CYCLE_TIME_S) - interface.layer.pc_stats.n_tx()).abs(),
        //     (TEST_DURATION * 500) as f64,
        //     "Not enough packts sent by pc"
        // );
    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */
        let (interface, mut reader, mut writer) = HidInterface::new();

        interface.layer.print();

        let reader_handle = Builder::new()
            .name("HID Reader".to_string())
            .spawn(move || {
                reader.pipeline();
            })
            .unwrap();

        let writer_handle = Builder::new()
            .name("HID Writer".to_string())
            .spawn(move || {
                writer.pipeline();
            })
            .unwrap();

        let interface_sim = Builder::new()
            .name("HID Control".to_string())
            .spawn(move || {
                sim_interface(interface);
            })
            .unwrap();

        reader_handle.join().expect("HID Reader failed");
        interface_sim.join().expect("HID Control failed");
        writer_handle.join().expect("HID Writer failed");
    }
}
