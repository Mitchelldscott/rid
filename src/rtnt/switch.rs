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

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use crate::{
    rtnt::*, 
};

/// The switch object
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub struct RTSwitch {
    n_outputs: u8,
}

impl RTNTask for RTSwitch {
    fn default() -> RTSwitch { RTSwitch { n_outputs: 1 } }

    fn size(&self) -> usize {

        self.n_outputs as usize
    
    }

    fn run(&mut self, input: &[f32]) -> TaskData { 

        let mut output = [0.0f32; MAX_TASK_DATA_FLOATS];
        
        if input[0] > 0.0 {

            for i in 0..self.n_outputs as usize {
                output[i] = input[i+1];
            }
        
        }

        output
    }



    fn configure(&mut self, data: &[TaskBuffer]) -> bool { 

        self.n_outputs = data[0][0];

        (self.n_outputs as usize) < MAX_TASK_DATA_FLOATS - 1

    }

    fn deconfigure(&self, data: &mut [TaskBuffer]) -> usize { 

        data[0][0] = self.n_outputs;
        1
    }
}




