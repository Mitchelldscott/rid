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

/////////////////////////////////////////////////////////////

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

/////////////////////////////////////////////////////////////

/// Simple type alias for more readability
/// currently does not have any implementations
pub type RIDReport = [u8; RID_PACKET_SIZE];

/// HID packet size, tried going bigger and things broke
pub const RID_PACKET_SIZE: usize = 64;

/// Id of the task this packet is meant for
pub const RID_TASK_INDEX: usize = 0;  
/// Mode of the packet
pub const RID_MODE_INDEX: usize = 1; 
/// Bytes reserved for PTP
pub const RID_PTP_RESERVED_BYTES: usize = 16;


/////////////////////////////////////////////////////////////

pub mod ptp;
pub mod rtnt;

/// std build that uses hidapi, a unix C api wrapper
/// there are plenty of alternatives for hidapi, should check them out
/// also includes toml parsing
#[cfg(feature = "std")]
pub mod host;
