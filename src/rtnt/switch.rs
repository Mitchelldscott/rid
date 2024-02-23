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
//! # Real Time Task: Switch
//!
//!   This task provides a block that can switch
//! its output on and off using a 2nd signal.

use crate::{
    TaskBuffer,
    rtnt::{
        default::RTNTask, 
    }
};

/// The switch object
pub struct RTSwitch {}

impl RTNTask for RTSwitch {
    fn new() -> RTSwitch { RTSwitch {} }

    fn run(&mut self, input: &[u8], output: &mut [u8]) { 
        
        if input[0] > 0 {

            output[..].copy_from_slice(&input[1..]);
        
        }
    }

    fn configure(&mut self, _: &[TaskBuffer]) -> bool { true }
}