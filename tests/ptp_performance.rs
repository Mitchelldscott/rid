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

use rid::host::{layer::*};
use rid::*;
use rid::ptp::*;

#[allow(dead_code)]
const VERBOSITY: usize = 1;
pub static TEST_DURATION: f32 = 10.0;

pub mod ptp_performance {

    use super::*;

    pub fn least_squares_conversion(out_time: &Vec<f32>, in_time: &Vec<f32>) -> (f32, f32) {

        let n = in_time.len() as f32;
        
        let (sigma, sigma_bar) = in_time.iter().fold((0.0, 0.0), |(sum, sum_sq), &t| ( sum + t, sum_sq + (t*t) ));
        
        let gamma = (n * sigma_bar) - (sigma * sigma);

        let (m, b) = out_time
            .iter()
            .zip(in_time)
            .fold((0.0, 0.0), |(m, b), (&to, ti)| ( 
                m + ((n * to * ti) - sigma),
                b + (to * (sigma_bar - (ti * sigma)))
            ));

        (m / gamma, b / gamma)
    }

    pub fn demo_rid(layer: &mut RIDLayer) {

        println!("[HID-Control]: Live");

        let mut local_offset = vec![];

        let mut t = Instant::now();
        let mut packet_flight_time = vec![];

        let mut host_offset_error: Vec<f32> = vec![];
        let mut client_offset_error: Vec<f32> = vec![];

        let mut host_truth: Vec<f32> = vec![];
        let mut client_truth: Vec<f32> = vec![];

        let mut client_prediction: Vec<f32> = vec![];

        let mut m = 1.0;
        let mut b = 0.0;

        let mut cr_min = f32::MAX;
        let mut cr_max = 0.0;
        let mut hr_min = f32::MAX;
        let mut hr_max = 0.0;

        layer.connected = true;

        while layer.system_time.time() < TEST_DURATION && layer.connected
        {


            let micros = layer.system_time.add_micros(t.elapsed().as_micros() as u32);
            t = Instant::now();

            let mut buffer = [0; RID_PACKET_SIZE];
            buffer[RID_MODE_INDEX] = 255;
            buffer[RID_TOGL_INDEX] = 255;


            match layer.read(&mut buffer) {

                RID_PACKET_SIZE => {

                    if micros > 2_000 {

                        let offset = layer.ptp_stamp.offset(); // calculates the current offset

                        let (cr, cw, hr, hw) = layer.ptp_stamp.marks();

                        let cr_s = cr as f32;
                        let hr_s = hr as f32;

                        if cr_s < cr_min { 
                            cr_min = cr_s;
                        }

                        if cr_s > cr_max {
                            cr_max = cr_s;
                        }
                        
                        if hr_s < hr_min { 
                            hr_min = hr_s;
                        }

                        if hr_s > hr_max {
                            hr_max = hr_s;
                        }

                        m = (cr_max - cr_min) / (hr_max - hr_min);
                        b = cr_max - ((cr_max - cr_min) / (hr_max - hr_min) * hr_max);

                        let host_measure = hr as f32;
                        let cl_offset = cw as f32 + offset;

                        let client_measure = cr as f32;
                        let ho_offset = hw as f32 - offset;
                        
                        local_offset.push(offset / 1_000_000.0);

                        host_truth.push(host_measure);
                        client_truth.push(client_measure / 1_000_000.0);

                        host_offset_error.push(client_measure - ho_offset);
                        client_offset_error.push(host_measure - cl_offset);

                        packet_flight_time.push(host_measure - (hw as f32));

                        client_prediction.push(((m * TEST_DURATION * 1_000_000.0) + b) / 1_000_000.0); // predict client time at 10s local time

                        if (local_offset.len()-1) % 10_000 == 0 {
                            println!("\n\t[PTP-DEMO]\tC(t) = {m} * H(t) + {b}");
                            println!("\tHost (s)\t\tClient (s)\t\tConversion Error <offset, pred> (us)");
                        }


                        if local_offset.len() % 250 == 0 {
                            println!("  {:.4}\t\t{:.4}\t\t{:.0}\t{:.0}", 
                                host_measure / 1_000_000.0,
                                client_measure / 1_000_000.0,
                                client_measure - ho_offset,
                                client_measure - ((m * host_measure) + b),
                            );
                        }
                    }
                }
                _ => {}
            }

            layer.write(&mut buffer);
            // layer.delay(t);

            if layer.delay(t) > TEENSY_CYCLE_TIME_US {
                println!("HID Control over cycled {}", t.elapsed().as_micros());
            }
        }

        // interface.layer.control_flags.shutdown();
        println!("[HID-Control]: shutdown");
        // interface.print();

        let ptp_mean =
            local_offset.iter().sum::<f32>() / local_offset.len() as f32;
        let ptp_std = (local_offset
            .iter()
            .map(|offset| (offset - ptp_mean) * (offset - ptp_mean))
            .sum::<f32>()
            / local_offset.len() as f32)
            .sqrt();
        
        let host_prediction = host_truth.iter().map(|&x| ((m * x) + b) / 1_000_000.0).collect::<Vec<f32>>();
        let host_scaled = host_truth.iter().map(|&x| x / 1_000_000.0).collect::<Vec<f32>>();

        println!(
            "PTP Offset stats: \n\tSamples: {}\n\t(mean, std): ({ptp_mean:.3}, {ptp_std:.3}) s\n\tHOST elapsed time: {} s\n\tMCU elapsed time: {} s",
            local_offset.len(),
            (hr_max - hr_min) / 1_000_000.0,
            (cr_max - cr_min) / 1_000_000.0,
        );

        assert_le!(ptp_std, 0.5, "PTP offset STD was too large");
        assert_le!(cr_min, cr_max, "MCU Min and Max time is invalid");
        assert_le!((TEST_DURATION - ((cr_max - cr_min) / 1_000_000.0)).abs(), 0.2, "Time elapsed differs on MCU");
        assert_le!((TEST_DURATION - ((hr_max - hr_min) / 1_000_000.0)).abs(), 0.2, "Time elapsed differs on HOST");

        let x = (0..local_offset.len())
            .map(|x| x as f32)
            .collect::<Vec<f32>>();

        let mut fg = Figure::new();
        let mut fg1 = Figure::new();
        let mut fg2 = Figure::new();
        let mut fg3 = Figure::new();

        fg.axes2d()
            .lines(
                &x,
                &local_offset,
                &[Caption("PTP Offset (us)"), Color("black")],
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
                &(host_offset_error),
                &[Caption("host conversion error usec (client time -> host time)"), Color("red")],
            )
            .lines(
                &x,
                &(client_offset_error),
                &[Caption("client conversion error usec"), Color("blue")],
            );
        


        fg2.axes2d()
            .lines(
                &x,
                &packet_flight_time,
                &[Caption("packet flight time (us)"), Color("black")],
            );

        fg3.axes2d()
            .lines(
                &host_scaled,
                &host_prediction,
                &[Caption("host prediction"), Color("red")],
            )
            .lines(
                &host_scaled,
                &client_truth,
                &[Caption("client measured"), Color("blue")],
            )
            .lines(
                &host_scaled,
                &client_prediction,
                &[Caption("client 10s prediction"), Color("green")],
            );

        
        let _ = fg.show();
        fg.close();

        let _ = fg1.show();
        fg1.close();

        let _ = fg2.show();
        fg2.close();

        let _ = fg3.show();
        fg3.close();
    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */
        let mut layer = RIDLayer::new(TEENSY_DEFAULT_VID, TEENSY_DEFAULT_PID, TEENSY_CYCLE_TIME_US);

        demo_rid(&mut layer);
    }
}
