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
use rid::ptp::*;

#[allow(dead_code)]
const VERBOSITY: usize = 1;
pub static TEST_DURATION: f32 = 3600.0;

pub mod ptp_performance {

    use super::*;

    pub fn demo_rid(interface: HidInterface, reader: &mut HidReader, writer: &mut HidWriter) {
        while !interface.layer.control_flags.is_connected() {}

        println!("[HID-Control]: Live");

        let mut local_offset = vec![];
        let mut ptp_stamp = TimeStamp::new(0, 0, 0, 0);

        let mut client_reads = vec![];
        let mut host_reads = vec![];
        // let mut client_writes = vec![];

        let mut t = Instant::now();
        let mut system_time = rid::ptp::Duration::default();

        let mut packet_flight_time = vec![];
        let mut predicted_hr = vec![];
        let mut predicted_cr = vec![];

        while system_time.time() < TEST_DURATION && !interface.layer.control_flags.is_shutdown()
        {
            let loopt = Instant::now();

            if interface.layer.control_flags.is_connected() {

                let mut buffer = [0; RID_PACKET_SIZE];
                buffer[RID_MODE_INDEX] = 255;
                buffer[RID_TOGL_INDEX] = 255;

                let millis = system_time.add_micros(t.elapsed().as_micros() as i32);
                t = Instant::now();

                match reader.read_raw() {
                    (Some(buffer), _datetime) => {

                        ptp_stamp.host_read(&buffer, millis);

                        if system_time.millis() > 1 {

                            let offset = ptp_stamp.offset() as f64; // calculates the current offset

                            let (cr, cw, hr, hw) = ptp_stamp.marks();

                            let hr_act_time = hr as f64 / 1_000.0;
                            let hr_pred_time = (cw as f64 + offset) / 1_000.0;

                            let cr_act_time = cr as f64 / 1_000.0;
                            let cr_pred_time = (hw as f64 - offset) / 1_000.0;
                            
                            local_offset.push(offset);

                            predicted_hr.push(hr_pred_time);
                            predicted_cr.push(cr_pred_time);

                            client_reads.push(cr_act_time);
                            host_reads.push(hr_act_time);

                            packet_flight_time.push(hr_act_time - (hw as f64 / 1_000.0));

                            if (local_offset.len()-1) % 10_000 == 0 {
                                println!("\n\t[PTP-INFO] {hr_act_time:.3} (s)\n\t  Local\t  MCU");
                            }

                            if local_offset.len() % 250 == 0 {
                                println!("\t{:.5}\t{:.5}", 
                                    hr_act_time - hr_pred_time,
                                    cr_act_time - cr_pred_time,
                                );
                            }
                        }
                    }
                    _ => {}
                }

                ptp_stamp.host_stamp(&mut buffer, millis);
                writer.write(&mut buffer);
            }

            interface.layer.delay(loopt);
            // if interface.layer.delay(loopt) > TEENSY_CYCLE_TIME_US {
            //     println!("HID Control over cycled {}", t.elapsed().as_micros());
            // }
        }

        interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        interface.print();

        let ptp_mean =
            local_offset.iter().sum::<f64>() / local_offset.len() as f64;
        let ptp_std = (local_offset
            .iter()
            .map(|offset| (offset - ptp_mean) * (offset - ptp_mean))
            .sum::<f64>()
            / local_offset.len() as f64)
            .sqrt();

        let mut cr_max = 0.0;
        let mut cr_min = f64::MAX;
        client_reads[1..].iter().for_each(|x| {
            if *x < cr_min { 
                cr_min = *x;
            }

            if *x > cr_max {
                cr_max = *x;
            }
        });

        let mut hr_max = 0.0;
        let mut hr_min = f64::MAX;
        host_reads[1..].iter().for_each(|x| {
            if *x < hr_min { 
                hr_min = *x;
            }

            if *x > hr_max {
                hr_max = *x;
            }
        });

        println!(
            "PTP Offset stats: \n\tSamples: {}\n\t(mean, std): ({ptp_mean:.3}, {ptp_std:.3}) ms\n\tHOST elapsed time: {}\n\tMCU elapsed time: {}",
            local_offset.len(),
            hr_max - hr_min,
            cr_max - cr_min,
        );

        assert_le!(ptp_std, 500.0, "PTP offset STD was too large");
        assert_le!((TEST_DURATION as f64 - (cr_max - cr_min)).abs(), 0.2, "Time elapsed differs on MCU");
        assert_le!((TEST_DURATION as f64 - (hr_max - hr_min)).abs(), 0.2, "Time elapsed differs on HOST");

        let x = (0..local_offset.len())
            .map(|x| x as f64)
            .collect::<Vec<f64>>();

        let mut fg = Figure::new();
        let mut fg1 = Figure::new();
        let mut fg2 = Figure::new();
        let mut fg3 = Figure::new();

        fg.axes2d()
            .lines(
                &x,
                &local_offset,
                &[Caption("PTP Offset (ms)"), Color("black")],
            )
            .lines(
                &x,
                &vec![ptp_mean; local_offset.len()],
                &[Caption("Average"), Color("green")],
            )
            .lines(
                &x,
                &vec![ptp_mean + ptp_std; local_offset.len()],
                &[Caption("2 Sigma bound"), Color("red")],
            )
            .lines(
                &x,
                &vec![ptp_mean - ptp_std; local_offset.len()],
                &[Color("red")],
            );
        
        fg1.axes2d()
            .lines(
                &x,
                &(host_reads.iter().zip(predicted_hr).map(|(&a, b)| a - b).collect::<Vec<f64>>()),
                &[Caption("host read prediction error (client time -> host time)"), Color("black")],
            );
        
        fg2.axes2d()
            .lines(
                &x,
                &(client_reads.iter().zip(predicted_cr).map(|(&a, b)| a - b).collect::<Vec<f64>>()),
                &[Caption("client read prediction error"), Color("black")],
            );


        fg3.axes2d()
            .lines(
                &x,
                &packet_flight_time,
                &[Caption("packet flight time"), Color("black")],
            );

        
        let _ = fg.show();
        fg.close();

        let _ = fg1.show_and_keep_running();
        let _ = fg2.show();
        fg1.close();
        fg2.close();

        let _ = fg3.show();
        fg3.close();


        

    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */
        let (interface, mut reader, mut writer) = HidInterface::new();

        interface.layer.print();

        demo_rid(interface, &mut reader, &mut writer);
    }
}
