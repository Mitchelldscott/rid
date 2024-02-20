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
//! Doc Template:
//!
//! [short sentence explaining what it is]
//! 
//! [more detailed explanation]
//! 
//! [at least one code example that users can copy/paste to try it]
//! 
//! [even more advanced explanations if necessary]
//! 
//! # Building
//! 
//! RID only supports running tests on the host.
//! 
//! 'cargo test --features="std" <test_name>'
//!
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

/// Simple type alias for more readability
/// currently does not have any implementations
pub type RIDReport = [u8; RID_PACKET_SIZE];

/// HID packet size, tried going bigger and things broke
pub const RID_PACKET_SIZE: usize = 64;

/// Mode index, 255 = init data, 1 = overwrite data, 13 = kill
pub const RID_MODE_INDEX: usize = 0;  
/// Toggle index (alt), init data: (1 = init task, 2 = config task) overwrite data: (latch)
pub const RID_TOGL_INDEX: usize = 1; 

/// Messaging rate for both host and client in seconds
pub const RID_CYCLE_TIME_S: f64 = 0.001;
/// Messaging rate for both host and client in milliseconds
pub const RID_CYCLE_TIME_MS: f64 = RID_CYCLE_TIME_S * 1E3;
/// Messaging rate for both host and client in microseconds
pub const RID_CYCLE_TIME_US: f64 = RID_CYCLE_TIME_S * 1E6;

/// Defualt Dyse Indstries vendor id
pub const RID_DEFAULT_VID: u16 = 0x1331;
/// Defualt Dyse Indstries product id
pub const RID_DEFAULT_PID: u16 = 0x0001;

pub mod ptp;

#[cfg(feature = "std")]
/// std build that uses hidapi, a unix C api wrapper
/// there are plenty of alternatives for hidapi, should check them out
pub mod host;
