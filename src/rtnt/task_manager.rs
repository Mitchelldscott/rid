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
    RTNT_DATA_INDEX, RTNT_HDR_INDEX, MAX_TASK_INPUTS,
    RIDReport, TaskBuffer, InputIDBuffer,
    rtnt::task_generator::*,
};

/// Specifies the state a Task is in
/// and the action required by the [TaskManager]
#[derive(PartialEq, Eq, Debug)]
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
#[derive(Debug, Eq, PartialEq)]
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
            total_chunks: 1,
            missing_chunks: [false; MAX_TASK_CONFIG_CHUNKS],
            buffer: [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS],
        }
    }

    pub fn new(total_chunks: usize, buffer: [TaskBuffer; MAX_TASK_CONFIG_CHUNKS]) -> TaskConfig {
        
        let id = 0;
        let missing_chunks = core::array::from_fn(|i| i < total_chunks);
        
        TaskConfig {
            id,
            total_chunks,
            missing_chunks,
            buffer,
        }
    }

    pub fn chunks(&self) -> usize {
        self.total_chunks
    }

    /// reset the chunks
    pub fn reset_chunks(&mut self) {
        
        self.missing_chunks = core::array::from_fn(|i| i < self.total_chunks);

    }

    /// clear the chunks
    pub fn clear_chunks(&mut self) {
        
        self.total_chunks = 0;
        self.missing_chunks = [false; MAX_TASK_CONFIG_CHUNKS];
        (0..MAX_TASK_CONFIG_CHUNKS).for_each(|i| (0..MAX_TASK_DATA_BYTES).for_each(|j| self.buffer[i][j] = 0)); 

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
    pub fn collect_chunk(&mut self, buffer: &[u8]) {

        let id = buffer[RTNT_DATA_INDEX];
        let chunk_num = buffer[RTNT_DATA_INDEX+2] as usize;
        let chunk = &buffer[RTNT_DATA_INDEX+3..MAX_TASK_DATA_BYTES+RTNT_DATA_INDEX+3];
        
        self.total_chunks = buffer[RTNT_DATA_INDEX+1] as usize;

        match self.id == id {
            true => {

                self.new_chunk(chunk_num, chunk);

            }
            _ => {

                self.id = id;
                self.reset_chunks();
                self.new_chunk(chunk_num, chunk);

            },
        }
    }

    /// Copy the chunk status from the buffer
    pub fn collect_status(&mut self, buffer: &[u8]) {

        self.id = buffer[RTNT_DATA_INDEX];        
        self.total_chunks = buffer[RTNT_DATA_INDEX+1] as usize;

        for i in 0..self.total_chunks {
            self.missing_chunks[i] = buffer[RTNT_DATA_INDEX+3+i] != 0;
        }

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
#[derive(Debug)]
pub struct TaskNode {
    /// Speceifies if the data should be streamed
    pub stream: bool,

    /// Rate this task will execute at
    pub rate: u16,
    /// Input Tasks (max tasks = 32, each bit specifies the input)
    pub inputs: InputIDBuffer,

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
            inputs: [0u8; MAX_TASK_INPUTS],

            data: TaskConfig::default(),
            status: TaskStatus::Configuration,
            driver: None,

            task: None,
        }
    }

    pub fn new(stream: bool, rate: u16, inputs: InputIDBuffer, driver: TaskDriver, data: TaskConfig) -> TaskNode {
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

    pub fn link(&mut self, link: InputIDBuffer) {
        self.inputs = link;
    }

    pub fn load_header(&mut self, buffer: &[u8]) -> TaskDriver {
        
        self.stream = buffer[RTNT_HDR_INDEX] > 0;
        self.rate = u16::from_be_bytes([buffer[RTNT_HDR_INDEX+2], buffer[RTNT_HDR_INDEX+3]]);
        self.inputs.copy_from_slice(&buffer[RTNT_HDR_INDEX+4..RTNT_HDR_INDEX+8]);

        TaskDriver::new(buffer[RTNT_HDR_INDEX+1])
    }

    pub fn dump_header(&self, buffer: &mut [u8]) {
        
        buffer[RTNT_HDR_INDEX] = self.stream as u8;
        buffer[RTNT_HDR_INDEX+1] = match &self.driver { Some(driver) => driver.as_u8(), None => 0, };
        buffer[RTNT_HDR_INDEX+2..RTNT_HDR_INDEX+4].copy_from_slice(&self.rate.to_be_bytes());
        buffer[RTNT_HDR_INDEX+4..RTNT_HDR_INDEX+8].copy_from_slice(&self.inputs);

    }

    pub fn init(&mut self, buffer: &[u8]) {

        let new_driver = self.load_header(buffer);

        match &mut self.driver {
            Some(driver) => {
                
                if *driver != new_driver {

                    self.data.reset_chunks();
                    *driver = new_driver;
                    self.task = None;

                }
            }
            None => {

                self.task = Some(TaskExecutable::generate(&new_driver));
                self.driver = Some(new_driver);
                
            },
        }
            
    }

    pub fn collect(&mut self, buffer: &[u8]) -> bool {

        let mut broadcast = false;
        let msg_status = TaskStatus::new(buffer[RID_TOGL_INDEX]);

        match msg_status {
            TaskStatus::Panic => {

                println!("Panic received");
                match self.status {
                    TaskStatus::Configuration | TaskStatus::Active => {
                        self.data.clear_chunks();
                        self.driver = None;
                        self.task = None;
                        self.status = TaskStatus::Configuration;
                    },
                    _ => {},
                };
                

            },

            TaskStatus::Active => {

                self.status = TaskStatus::Standby;
                self.data.clear_chunks();
                // publish data here if stream is true;

                println!("Activation Received\tnode: {}\tmissing chunks: {}", buffer[RID_MODE_INDEX]-1, self.data.missing_chunks());
                if self.stream {
                    broadcast = true;
                }

            }

            TaskStatus::Standby => {

                match self.status {
                    TaskStatus::Configuration => {
                        self.init(buffer);
                        self.status = TaskStatus::Configuration;

                        self.data.collect_chunk(buffer);
                        println!("Standby Received\tnode: {}\tmissing chunks: {}", buffer[RID_MODE_INDEX]-1, self.data.missing_chunks());
                    }

                    _ => {},
                }
            }

            TaskStatus::Configuration => {

                match self.status {

                    TaskStatus::Standby => {

                        self.data.collect_status(&buffer);
                        println!("Configuration Received\tnode: {}\tmissing chunks: {}", buffer[RID_MODE_INDEX]-1, self.data.missing_chunks());
                    }

                    _ => {},
                }

            }
        }

        broadcast
    }


    pub fn emit(&mut self, id: u8) -> Option<RIDReport> {

        let mut buffer = [0u8; RID_PACKET_SIZE];
        buffer[RID_MODE_INDEX] = id + 1;
        buffer[RID_TOGL_INDEX] = self.status.as_u8();

        self.dump_header(&mut buffer);

        match self.status {
            TaskStatus::Standby => {
                match self.data.missing_chunks() > 0 {
                    true => {
                        println!("Emitting Standby\tnode: {id}\tmissing chunks: {}", self.data.missing_chunks());
                        self.data.emit_chunk(&mut buffer);
                        Some(buffer)
                    }
                    false => None,
                }
            }

            TaskStatus::Configuration => {

                if self.data.missing_chunks() == 0 {
                    match &mut self.task {
                        Some(task) => {
                            if task.configure(self.data.data()) {
                                self.status = TaskStatus::Active;
                            }
                        }
                        _ => {},
                    }
                }

                println!("Emitting Configuration\tnode: {id}\tmissing chunks: {}", self.data.missing_chunks());
                self.data.emit_status(&mut buffer);
                Some(buffer)
            }
            TaskStatus::Active => {

                match self.stream {
                    // publish data here if stream is true;
                    true => {
                        println!("Emitting Activity\tnode: {id}");
                        Some(buffer)
                    },
                    false => None,

                }

            },
            TaskStatus::Panic => {
                println!("Emitting Panic\tnode: {id}");
                Some(buffer)
            },
        }

    }
}

/// Stores and manages all tasks and their data
pub struct TaskManager {

    /// current publishing node
    pub context: usize,
    /// the number of active nodes
    pub n_nodes: usize,
    /// list of nodes
    pub nodes: [TaskNode; MAX_TASKS],
    /// buffer containing each tasks output data
    pub output_buffer: [Option<RIDReport>; MAX_TASKS],

}


impl TaskManager {

    /// Create a new [TaskManager] object
    pub fn default() -> TaskManager {

        TaskManager {

            context: 0,
            n_nodes: 0,
            nodes: core::array::from_fn(|_| TaskNode::empty()),
            output_buffer: [None; MAX_TASKS],
        
        }
    }

    pub fn init_node(&mut self, node: TaskNode) {

        self.nodes[self.n_nodes] = node;
        self.n_nodes += 1;

    }

    pub fn collect(&mut self, buffer: &RIDReport) -> Option<RIDReport> {

        let node_select = buffer[RID_MODE_INDEX] as usize;
        match node_select > 0 {
            true => {
                match node_select > self.n_nodes { true => self.n_nodes = node_select, false => {}, };
                match self.nodes[node_select-1].collect(buffer) {
                    true => { 
                        let mut publish_buffer = [0u8; RID_PACKET_SIZE];
                        publish_buffer.copy_from_slice(&buffer[..RID_PACKET_SIZE]);
                        Some(publish_buffer)
                    }
                    false => None,
                }
            },
            false => None,
        }
        
        
    }

    pub fn spin(&mut self) -> Option<RIDReport> {

        for i in 0..self.n_nodes {
            match self.nodes[i].driver {
                Some(_) => {
                    
                    self.output_buffer[i] = self.nodes[i].emit(i as u8);
                
                }
                None => self.output_buffer[i] = None,
            }
        }

       
        let mut ctr = 0;
        let mut output_buffer = [0u8; RID_PACKET_SIZE];

        while ctr < self.n_nodes {
        
            self.context = (self.context + 1) % self.n_nodes;

            match self.output_buffer[self.context] {
        
                Some(buffer) => {
                    output_buffer.copy_from_slice(&buffer[..RID_PACKET_SIZE]);
                    return Some(output_buffer);
                },
        
                None => {
                    ctr += 1;
                }
        
            }
        
        }

        None
    }

    pub fn panic(&mut self) -> Option<RIDReport> {

        for i in 0..self.n_nodes {
            self.nodes[i].data.clear_chunks();
            self.nodes[i].status = TaskStatus::Panic;
            self.output_buffer[i] = self.nodes[i].emit(i as u8);
        }

        let mut ctr = 0;
        let mut output_buffer = [0u8; RID_PACKET_SIZE];

        while ctr < self.n_nodes {
        
            self.context = (self.context + 1) % self.n_nodes;

            match self.output_buffer[self.context] {
        
                Some(buffer) => {
                    output_buffer.copy_from_slice(&buffer[..RID_PACKET_SIZE]);
                    return Some(output_buffer);
                },
        
                None => {
                    ctr += 1;
                }
        
            }
        
        }

        None
    }
}