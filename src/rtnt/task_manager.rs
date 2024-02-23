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
//! # Real Time Task Manager
//!
//!   This crate can be included in a firmware build (use the client calls) 
//! or built using the "std" feature.
//!

use crate::{
    MAX_TASKS,
    MAX_TASK_DATA_BYTES,
    MAX_TASK_CONFIG_CHUNKS,
    TaskBuffer,
    rtnt::task_generator::*,
};

/// Specifies the state a Task is in
/// and the action required by the [TaskManager]
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

/// Configuration data-structure for tasks.
///
/// Tasks need more config data than fits into
/// RID packets. This struct lets configurations
/// get sharded into chunks and shared. The struct
/// also notifies users when it has collected all
/// the chunks.
pub struct TaskConfig {
    /// Tells the struct when a new set of chunks is coming
    id: u8,
    /// Tells the struct when a new set of chunks is coming
    total_chunks: usize,
    /// Missing chunks of the configuration data
    missing_chunks: [bool; MAX_TASK_CONFIG_CHUNKS],
    /// The data
    data: [TaskBuffer; MAX_TASK_CONFIG_CHUNKS],
}


impl TaskConfig {
    /// Create an empty configuration missing all chunks
    pub fn new() -> TaskConfig {
        TaskConfig {
            id: u8::MAX,
            total_chunks: 0,
            missing_chunks: [true; MAX_TASK_CONFIG_CHUNKS],
            data: [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS],
        }
    }

    /// reset the chunks
    pub fn reset(&mut self) {
        
        self.missing_chunks = [true; MAX_TASK_CONFIG_CHUNKS];

    }

    /// Inserts chunk into data buffer
    pub fn new_chunk(&mut self, chunk_num: usize, chunk: &[u8]) {

        self.missing_chunks[chunk_num] = false;
        self.data[chunk_num].copy_from_slice(&chunk[..MAX_TASK_DATA_BYTES]);

    }

    /// Returns the number of missing chunks
    pub fn missing_chunks(&self) -> usize {
        
        let mut chunks = 0;

        for i in 0..MAX_TASK_CONFIG_CHUNKS {
            if !self.missing_chunks[i] {
                chunks += 1;
            }
        }

        self.total_chunks - chunks
    }

    /// Insert a new chunk or reset the current chunks
    pub fn collect(&mut self, data: &[u8]) -> usize {

        let id = data[0];
        let chunk_num = data[2] as usize;
        let chunk = &data[3..3+MAX_TASK_DATA_BYTES];
        
        self.total_chunks = data[1] as usize;

        match self.id == id {
            true => {

                self.new_chunk(chunk_num, chunk);

            }
            _ => {

                self.id = id;
                self.reset();
                self.new_chunk(chunk_num, chunk);

            },
        }

        self.missing_chunks()
    }

    pub fn buffer(&self) -> &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS] {

        &self.data
    
    }
}

/// Node containing an executable, stream, rate, inputs and status
pub struct TaskNode {

    /// Speceifies if the data should be streamed
    pub stream: bool,

    /// Rate this task will execute at
    pub rate: u16,
    /// Input Tasks (max tasks = 32, each bit specifies the input)
    pub inputs: u32,

    /// Buffer containing configuration data for the task
    pub data: TaskConfig,
    /// Status of the task, enables and disables running the task
    pub status: TaskStatus,

    /// Optional Executable, there is always the max number of [TaskNodes]
    /// but not all will have [TaskExecutable]s
    pub task: Option<TaskExecutable>,

}

impl TaskNode {
    /// Defualt constructor
    pub fn empty() -> TaskNode {
        TaskNode {

            stream: false,

            rate: 250,
            inputs: 0,

            data: TaskConfig::new(),
            status: TaskStatus::Configuration,

            task: None,
        }
    }

    pub fn init(&mut self, data: &[u8]) {

        let driver = TaskDriver::new(data[0]);

        self.stream = data[1] > 0;
        self.rate = u16::from_be_bytes([data[2], data[3]]);
        self.inputs = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        self.data.reset();
        self.task = Some(TaskExecutable::generate(driver));
            
    }

    pub fn collect(&mut self, data: &[u8]) -> bool {
        
        match self.data.collect(&data[..2+MAX_TASK_DATA_BYTES]) {
            0 => match &mut self.task {
                Some(task) => task.configure(self.data.buffer()),
                _ => false,
            }
            _ => false,
        }
    }

}

/// Stores and manages all tasks and their data
pub struct TaskManager {

    /// list of nodes
    nodes: [TaskNode; MAX_TASKS],
    /// buffer containing each tasks output data
    output_buffer: [Option<TaskBuffer>; MAX_TASKS],

}


impl TaskManager {

    /// Create a new [TaskManager] object
    pub fn new() -> TaskManager {

        TaskManager {

            nodes: core::array::from_fn(|_| TaskNode::empty()),
            output_buffer: [None; MAX_TASKS],
        
        }
    }

    pub fn initialize(&mut self, data: &[u8]) {

        self.nodes[data[0] as usize].init(&data[1..9]);

    }

    pub fn collect(&mut self, data: &[u8]) {


    }
}