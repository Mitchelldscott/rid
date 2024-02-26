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
//! # Real Time Numerical Tasks

use crate::{MAX_TASK_INPUTS, TaskBuffer};

/// Trait for RTNTasks, helps make sure they follow the convention (only useful for developers)
pub trait RTNTask {
    /// Create a new exectuable
    fn new() -> Self;
    /// Run the executable with a ref to an input buffer 
    /// and a mutable reference to the output buffer
    fn run(&mut self, input: [&[u8]; MAX_TASK_INPUTS], output: &mut [u8]);
    /// Once the configuration chunks have all arrived pass the buffe to this
    fn configure(&mut self, data: &[TaskBuffer]) -> bool;
    /// Convert the tasks data into TaskConfig
    fn deconfigure(&self, data: &mut [TaskBuffer]) -> usize;
}

/// Specifies the state a Task is in
/// and the action required by the [TaskManager]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskStatus {
    /// The task is panicing due to a runtime/configuration error
    Panic,
    /// The task is executing normally
    Active,
    /// The task is disable, but nothings wrong
    Standby,
    /// The task is awaiting configuration chunks
    Configuration,
}

impl TaskStatus {
    /// Create a new status from u8
    pub fn new(id: u8) -> TaskStatus {
        match id {
            0 => TaskStatus::Active,
            1 => TaskStatus::Standby,
            2 => TaskStatus::Configuration,           
            _ => TaskStatus::Panic,
        }
    }

    /// Convert Self to a u8
    pub fn as_u8(&self) -> u8 {
        match self {
            TaskStatus::Panic => 255,
            TaskStatus::Active => 0,
            TaskStatus::Standby => 1,
            TaskStatus::Configuration => 2,           
        }
    }
}

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum PacketType {
    /// init data for a [TaskNode]
    Init,
    /// a chunk of configuration data
    Chunk,
    /// the status of configuration data
    Status,
    /// data from a streamed task
    Data,
    /// kill
    Kill,
}

impl PacketType {
    /// Create a new type from u8
    pub fn new(id: u8) -> PacketType {
        match id {
            0 => PacketType::Init,
            1 => PacketType::Chunk,
            2 => PacketType::Status,           
            3 => PacketType::Kill,           
            _ => PacketType::Data,
        }
    }

    /// Convert Self to a u8
    pub fn as_u8(&self) -> u8 {
        match self {
            PacketType::Data => 255,
            PacketType::Init => 0,
            PacketType::Chunk => 1,
            PacketType::Status => 2,           
            PacketType::Kill => 3,           
        }
    }
}

pub mod task_generator;
pub mod task_manager;

pub mod switch;
pub mod constant;
