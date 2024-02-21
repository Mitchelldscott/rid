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
//! # Demonstrate and verify PTP implementation
//! The validation criteria for the [TimeStamp]
//! and [Duration] shows that the layer can
//! accurately convert from one time to the other.
//! This conversion must hold for offseting a current time:
//! C = H + o, as well as converting to a future time: C(t) = m * H(t) + b.
//! In this scenario there is not a constant offset between system times,
//! This means the calculated offset is invalid after the instant it is calculated.

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

use rid::{
    ptp::*,
    host::*,
    RID_PACKET_SIZE,
    RID_MODE_INDEX, RID_TOGL_INDEX,
    RID_DEFAULT_VID, RID_DEFAULT_PID,
    RID_CYCLE_TIME_S, RID_CYCLE_TIME_US,
};

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
        
        let mut packet_flight_time = vec![];

        let mut host_truth: Vec<f32> = vec![];
        let mut client_truth: Vec<f32> = vec![];

        let mut client_prediction: Vec<f32> = vec![];

        let mut host_ptp_error: Vec<f32> = vec![];
        let mut client_ptp_error: Vec<f32> = vec![];

        let mut write_count = 0.0;

        while layer.host_elapsed() / 1_000_000.0 < TEST_DURATION
        {

            let flight_time = layer.spin();
            
            if flight_time > 0.0 && layer.host_elapsed() > 1_000.0 {

                if write_count as u32 % (100 * TEST_DURATION as u32) == 0 {

                    layer.print_header();
                
                }

                if write_count as u32 % TEST_DURATION as u32 == 0 {
                    
                    local_offset.push(layer.ptp_offset());

                    let host_read = layer.ptp_stamp[2] as f32;
                    let host_write = layer.ptp_stamp[3] as f32;
                    let client_read = layer.ptp_stamp[0] as f32;
                    let client_write = layer.ptp_stamp[1] as f32;

                    let (ho_err, cl_err) = layer.print();

                    host_ptp_error.push(ho_err);
                    client_ptp_error.push(cl_err);

                    packet_flight_time.push(flight_time);

                    client_truth.push(layer.client_elapsed());
                    host_truth.push(layer.host_elapsed());

                    client_prediction.push(layer.linear_to_client(TEST_DURATION * 1_000_000.0) / 1_000_000.0);
                }

            }

            write_count += 1.0;

            layer.timestep();

        }

        println!("[HID-Control]: shutdown {}", layer.host_elapsed());

        let ptp_mean =
            local_offset.iter().sum::<f32>() / (local_offset.len() as f32);
        let ptp_std = (local_offset
            .iter()
            .map(|offset| (offset - ptp_mean) * (offset - ptp_mean))
            .sum::<f32>()
            / local_offset.len() as f32)
            .sqrt();

        let host_prediction = host_truth.iter().map(|&x| layer.linear_to_client(x) / 1_000_000.0).collect::<Vec<f32>>();
        let host_scaled = host_truth.iter().map(|&x| x / 1_000_000.0).collect::<Vec<f32>>();

        println!(
            "PTP Offset stats: \n\tSamples: {}\n\t(mean, std): ({ptp_mean:.3}, {ptp_std:.3}) s\n\tHOST elapsed time: {} s [{}, {}]\n\tMCU elapsed time: {} s [{}, {}]",
            local_offset.len(),
            layer.host_elapsed() / 1_000_000.0,
            layer.host_start / 1_000_000.0,
            layer.ptp_stamp[2] as f32 / 1_000_000.0,
            layer.client_elapsed() / 1_000_000.0,
            layer.client_start / 1_000_000.0,
            layer.ptp_stamp[1] as f32 / 1_000_000.0,
        );

        let x = (0..local_offset.len())
            .map(|x| x as f32 * TEST_DURATION)
            .collect::<Vec<f32>>();

        let mut fg = Figure::new();
        let mut fg1 = Figure::new();
        let mut fg2 = Figure::new();
        let mut fg3 = Figure::new();

        fg.axes2d()
            .lines(
                &x,
                &local_offset,
                &[Caption("PTP Offset (microseconds)"), Color("black")],
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
                &(host_ptp_error),
                &[Caption("Cr(t) - Hw(t) + o(t) (microseconds)"), Color("red")],
            )
            .lines(
                &x,
                &(client_ptp_error),
                &[Caption("Hw(t) - Cr(t) - o(t)"), Color("blue")],
            );
        


        fg2.axes2d()
            .lines(
                &x,
                &packet_flight_time,
                &[Caption("flight time (microseconds)"), Color("black")],
            );

        fg3.axes2d()
            .lines(
                &host_scaled,
                &host_prediction,
                &[Caption("C(t) = m * H(t) + b"), Color("red")],
            )
            .lines(
                &host_scaled,
                &client_truth.iter().map(|x| x / 1_000_000.0).collect::<Vec<f32>>(),
                &[Caption("C(t) (seconds)"), Color("blue")],
            )
            .lines(
                &host_scaled,
                &client_prediction,
                &[Caption("C = (m * duration) + b"), Color("green")],
            );

        
        let _ = fg.show();
        let _ = fg.save_to_png("~/RoPro/rid/doc/ptp_results/offset.png", 800, 500);
        fg.close();

        let _ = fg1.show();
        let _ = fg1.save_to_png("~/RoPro/rid/doc/ptp_results/ptp_offset_err.png", 800, 500);
        fg1.close();

        let _ = fg2.show();
        let _ = fg2.save_to_png("~/RoPro/rid/doc/ptp_results/flight_time.png", 800, 500);
        fg2.close();

        let _ = fg3.show();
        let _ = fg3.save_to_png("~/RoPro/rid/doc/ptp_results/linear_conv.png", 800, 500);
        fg3.close();

        assert_le!(0.9, write_count / (TEST_DURATION as f64 / RID_CYCLE_TIME_S), "Insufficient writes to client");
        assert_le!(ptp_std / 1_000_000.0, TEST_DURATION / 175.0, "PTP offset STD was too large");
        assert_le!(0.0, layer.client_elapsed(), "MCU elapsed time is invalid");
        assert_le!(0.98, (layer.client_elapsed() / 1_000_000.0) / TEST_DURATION, "Time elapsed differs on MCU");
        assert_le!(0.98, (layer.host_elapsed() / 1_000_000.0) / TEST_DURATION, "Time elapsed differs on HOST");
    }

    #[test]
    pub fn hid_spawner() {
        /*
            Start an hid layer
        */
        let mut layer = RIDLayer::new(RID_DEFAULT_VID, RID_DEFAULT_PID);

        demo_rid(&mut layer);
    }
}
