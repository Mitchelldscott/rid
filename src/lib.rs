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

#![cfg_attr(not(feature = "std"), no_std)]
// #![warn(missing_docs)]

pub const RID_PACKET_SIZE: usize = 64;

pub const RID_MODE_INDEX: usize = 0; // 255 = init data, 1 = overwrite data, 13 = kill
pub const RID_TOGL_INDEX: usize = 1; // init data: (1 = init task, 2 = config task) overwrite data: (latch)
pub const RID_TASK_INDEX: usize = 2; // only applies to init/overwrite data
pub const RID_DATA_INDEX: usize = 3; // data start

pub type RIDReport = [u8; RID_PACKET_SIZE];


pub mod ptp;

#[cfg(feature = "std")]
pub mod host;

pub struct RTTask {

}