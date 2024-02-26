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
//! # Real Time Task: Constant value
//!
//!   This task provides a block that can switch
//! its output on and off using a 2nd signal.

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use crate::{
    TaskBuffer,
    MAX_TASK_INPUTS,
    rtnt::RTNTask, 
};

/// The switch object
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct RTConstant {
    value: f32,
}

impl RTNTask for RTConstant {
    fn new() -> RTConstant { RTConstant { value: 0.0 } }

    fn run(&mut self, input: [&[u8]; MAX_TASK_INPUTS], output: &mut [u8]) { 
        
        
        output[0..4].copy_from_slice(&self.value.to_be_bytes());

    }

    fn configure(&mut self, buffer: &[TaskBuffer]) -> bool { 

        self.value = f32::from_be_bytes([buffer[0][0], buffer[0][1], buffer[0][2], buffer[0][3]]);

        true

    }

    fn deconfigure(&self, buffer: &mut [TaskBuffer]) -> usize { 

        buffer[0][0..4].copy_from_slice(&self.value.to_be_bytes());

        1

    }
}