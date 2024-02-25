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
//! # Real Time Task: Default
//!
//!   This task provides a block that copies its inputs
//! to its output buffer.

use crate::TaskBuffer;

/// Trait for RTNTasks, helps make sure they follow the convention (only useful for developers)
pub trait RTNTask {
    /// Create a new exectuable
    fn new() -> Self;
    /// Run the executable with a ref to an input buffer 
    /// and a mutable reference to the output buffer
    fn run(&mut self, input: &[u8], output: &mut [u8]);
    /// Once the configuration chunks have all arrived pass the buffe to this
    fn configure(&mut self, data: &[TaskBuffer]) -> bool;
}