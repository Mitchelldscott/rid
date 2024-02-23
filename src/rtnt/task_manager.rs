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
    MAX_TASKS, MAX_TASK_DATA_BYTES, MAX_TASK_CONFIG_CHUNKS, 
    RID_PACKET_SIZE, RID_MODE_INDEX, RID_TOGL_INDEX,
    RTNT_DATA_INDEX, RTNT_HDR_INDEX,
    RIDReport, TaskBuffer,
    rtnt::task_generator::*,
};

/// Specifies the state a Task is in
/// and the action required by the [TaskManager]
#[derive(PartialEq, Eq)]
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
    pub fn new(id: u8) -> TaskStatus {
        match id {
            0 => TaskStatus::Active,
            1 => TaskStatus::Standby,
            2 => TaskStatus::Configuration,           
            _ => TaskStatus::Panic,
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            TaskStatus::Panic => 255,
            TaskStatus::Active => 0,
            TaskStatus::Standby => 1,
            TaskStatus::Configuration => 2,           
        }
    }
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
    buffer: [TaskBuffer; MAX_TASK_CONFIG_CHUNKS],
}


impl TaskConfig {
    /// Create an empty configuration missing all chunks
    pub fn default() -> TaskConfig {
        TaskConfig {
            id: u8::MAX,
            total_chunks: 0,
            missing_chunks: [true; MAX_TASK_CONFIG_CHUNKS],
            buffer: [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS],
        }
    }

    pub fn new(total_chunks: usize, buffer: [TaskBuffer; MAX_TASK_CONFIG_CHUNKS]) -> TaskConfig {
        TaskConfig {
            id: 0,
            total_chunks: 0,
            missing_chunks: [true; MAX_TASK_CONFIG_CHUNKS],
            buffer: buffer,
        }
    }

    /// reset the chunks
    pub fn reset(&mut self) {
        
        self.missing_chunks = core::array::from_fn(|i| i < self.total_chunks);

    }

    /// Inserts chunk into data buffer
    pub fn new_chunk(&mut self, chunk_num: usize, chunk: &[u8]) {

        self.missing_chunks[chunk_num] = false;
        self.buffer[chunk_num].copy_from_slice(&chunk[..MAX_TASK_DATA_BYTES]);

    }

    /// Returns the number of missing chunks
    pub fn missing_chunks(&self) -> usize {
        
        let mut chunks = 0;

        for i in 0..self.total_chunks {
            if self.missing_chunks[i] {
                chunks += 1;
            }
        }

        chunks
    }

    pub fn first_missing(&self) -> Option<usize> {
        for i in 0..self.total_chunks {
            if self.missing_chunks[i] {
                return Some(i);
            }
        }
        None
    }

    /// Insert a new chunk or reset the current chunks
    pub fn collect_chunk(&mut self, data: &[u8]) -> usize {

        let id = data[RTNT_DATA_INDEX];
        let chunk_num = data[RTNT_DATA_INDEX+2] as usize;
        let chunk = &data[RTNT_DATA_INDEX+3..MAX_TASK_DATA_BYTES+RTNT_DATA_INDEX+3];
        
        self.total_chunks = data[RTNT_DATA_INDEX+1] as usize;

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

    /// Insert a new chunk or reset the current chunks
    pub fn collect_status(&mut self, buffer: &[u8]) -> usize {

        self.id = buffer[RTNT_DATA_INDEX];        
        self.total_chunks = buffer[RTNT_DATA_INDEX+1] as usize;

        for i in 0..self.total_chunks {
            self.missing_chunks[i] = buffer[RTNT_DATA_INDEX+3+i] != 0;
        }

        self.missing_chunks()
    }

    pub fn emit_chunk(&self, buffer: &mut [u8]) {

        match self.first_missing() {
            Some(chunk_num) => {
                buffer[RTNT_DATA_INDEX] = self.id;
                buffer[RTNT_DATA_INDEX+1] = self.total_chunks as u8;
                buffer[RTNT_DATA_INDEX+2] = chunk_num as u8;
                buffer[RTNT_DATA_INDEX+3..MAX_TASK_DATA_BYTES+RTNT_DATA_INDEX+3].copy_from_slice(&self.buffer[chunk_num]);
            }
            _ => {},
        }

       

    }

    pub fn emit_status(&self, buffer: &mut [u8]) {

        buffer[RTNT_DATA_INDEX] = self.id;
        buffer[RTNT_DATA_INDEX+1] = self.total_chunks as u8;
        buffer[RTNT_DATA_INDEX+2] = u8::MAX;

        for i in 0..self.total_chunks {
            buffer[RTNT_DATA_INDEX+3+i] = self.missing_chunks[i] as u8;
        }

    }


    pub fn data(&self) -> &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS] {

        &self.buffer
    
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
    /// The driver assigned to this node
    pub driver: Option<TaskDriver>,

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

            data: TaskConfig::default(),
            status: TaskStatus::Configuration,
            driver: None,

            task: None,
        }
    }

    pub fn new(stream: bool, rate: u16, inputs: u32, driver: TaskDriver, data: TaskConfig) -> TaskNode {
        TaskNode {

            stream: stream,

            rate: rate,
            inputs: inputs,

            data: data,
            status: TaskStatus::Configuration,
            driver: Some(driver),

            task: None,
        }
    }

    pub fn load_header(&mut self, buffer: &[u8]) -> TaskDriver {
        
        self.stream = buffer[RTNT_HDR_INDEX] > 0;
        self.rate = u16::from_be_bytes([buffer[RTNT_HDR_INDEX+2], buffer[RTNT_HDR_INDEX+3]]);
        self.inputs = u32::from_be_bytes([buffer[RTNT_HDR_INDEX+4], buffer[RTNT_HDR_INDEX+5], buffer[RTNT_HDR_INDEX+6], buffer[RTNT_HDR_INDEX+7]]);

        TaskDriver::new(buffer[RTNT_HDR_INDEX+1])
    }

    pub fn dump_header(&mut self, buffer: &mut [u8]) {
        
        buffer[RTNT_HDR_INDEX] = self.stream as u8;
        buffer[RTNT_HDR_INDEX+1] = match &self.driver { Some(driver) => driver.as_u8(), None => 0, };
        buffer[RTNT_HDR_INDEX+2..RTNT_HDR_INDEX+4].copy_from_slice(&self.rate.to_be_bytes());
        buffer[RTNT_HDR_INDEX+4..RTNT_HDR_INDEX+8].copy_from_slice(&self.inputs.to_be_bytes());

    }

    pub fn init(&mut self, buffer: &[u8]) {

        let new_driver = self.load_header(buffer);

        match &mut self.driver {
            Some(driver) => {
                
                if *driver != new_driver {

                    self.data.reset();
                    *driver = new_driver;
                    self.task = None;

                }
            }
            None => {

                self.data.reset();
                self.task = Some(TaskExecutable::generate(&new_driver));
                self.driver = Some(new_driver);
                
            },
        }
            
    }

    pub fn collect(&mut self, buffer: &[u8]) {

        let msg_status = TaskStatus::new(buffer[RID_TOGL_INDEX]);

        match msg_status {
            TaskStatus::Panic => {

                self.data.reset();
                self.driver = None;
                self.task = None;

            },

            TaskStatus::Active => {

                if self.status == TaskStatus::Standby {
                    self.status = TaskStatus::Active;
                }

            }

            TaskStatus::Standby => {

                self.init(buffer);
                self.status = TaskStatus::Configuration;

                if self.data.collect_chunk(buffer) == 0 {
                    match &mut self.task {
                        Some(task) => {
                            if task.configure(self.data.data()) {
                                self.status = TaskStatus::Standby;
                            }
                        }
                        _ => {},
                    }
                }

            }

            TaskStatus::Configuration => {

                if self.status != TaskStatus::Configuration {

                    self.data.collect_status(&buffer);

                }

            }
        }
    }


    pub fn emit(&self, buffer: &mut [u8]) {

        buffer[RID_TOGL_INDEX] = self.status.as_u8();

        match self.status {
            TaskStatus::Standby => {

                self.data.emit_chunk(buffer);

            }

            TaskStatus::Configuration => {

                self.data.emit_status(buffer);

            }
            _ => {},
        }

    }
}

/// Stores and manages all tasks and their data
pub struct TaskManager {

    /// list of nodes
    pub nodes: [TaskNode; MAX_TASKS],
    /// buffer containing each tasks output data
    pub output_buffer: [Option<RIDReport>; MAX_TASKS],

}


impl TaskManager {

    /// Create a new [TaskManager] object
    pub fn default() -> TaskManager {

        TaskManager {

            nodes: core::array::from_fn(|_| TaskNode::empty()),
            output_buffer: [None; MAX_TASKS],
        
        }
    }

    pub fn new(nodes: [TaskNode; MAX_TASKS]) -> TaskManager {

        TaskManager {

            nodes: nodes,
            output_buffer: [None; MAX_TASKS],
        
        }
    }

    pub fn collect(&mut self, buffer: &RIDReport) {

        self.nodes[buffer[RID_MODE_INDEX] as usize].collect(buffer);
    }

    pub fn spin(&mut self) {

        for i in 0..MAX_TASKS {
            match self.nodes[i].driver {
                Some(_) => {
                    let mut buffer = [0u8; RID_PACKET_SIZE];
                    buffer[RID_MODE_INDEX] = i as u8;
                    self.nodes[i].emit(&mut buffer);
                    self.output_buffer[i] = Some(buffer);
                }
                None => self.output_buffer[i] = None,
            }
            
        }
    }
}