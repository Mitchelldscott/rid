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
    rtnt::{task_generator::*, TaskStatus, PacketType},
};



/// Configuration data-structure for tasks.
///
/// Tasks need more config data than fits into
/// RID packets. This struct lets configurations
/// get sharded into chunks and shared. The struct
/// also notifies users when it has collected all
/// the chunks.
#[cfg_attr(feature = "std", derive(PartialEq, Eq, Debug))]
pub struct TaskConfig {
    /// Tells the struct when a new set of chunks is coming
    id: u8,
    // Unused config will have 0 chunks, all tasks have atleast 1 chunk
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

    /// Check if the buffer is being used
    pub fn is_init(&self) -> bool {

        self.total_chunks > 0
    
    }

    /// Get the number of chunks
    pub fn chunks(&self) -> usize {

        self.total_chunks
    
    }

    /// Reset the chunks
    /// 
    /// Clears data. Does not change number of chunks
    pub fn reset_chunks(&mut self) {
        
        self.missing_chunks = core::array::from_fn(|i| i < self.total_chunks);
        (0..MAX_TASK_CONFIG_CHUNKS).for_each(|i| (0..MAX_TASK_DATA_BYTES).for_each(|j| self.buffer[i][j] = 0)); 

    }

    

    /// Clear the chunks
    ///
    /// Clears the data and the number of chunks.
    /// This is required to make the buffer available for a
    /// new task. 
    pub fn clear_chunks(&mut self) {
        
        self.total_chunks = 0;
        self.reset_chunks();

    }

    /// Inserts chunk into data buffer and clear the missing flag
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

    /// Returns the index of the first missing chunk
    pub fn first_missing(&self) -> Option<usize> {
        for i in 0..self.total_chunks {
            if self.missing_chunks[i] {
                return Some(i);
            }
        }
        None
    }

    /// Insert a new chunk or reset the current chunks
    /// 
    /// If the id changes it indicates a new configuration
    /// is available. When a new configuration becomes available
    /// the buffer will reset, then consume the chunk.
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

    /// Copy the chunk status from the buffer.
    ///
    /// This will only happen on hosts trying to
    /// configure a set of tasks.
    pub fn collect_status(&mut self, buffer: &[u8]) -> usize {

        self.id = buffer[RTNT_DATA_INDEX];        

        for i in 0..self.total_chunks {
            self.missing_chunks[i] = buffer[RTNT_DATA_INDEX+3+i] != 0;
        }

        self.missing_chunks()

    }

    /// Copy the configuration data to a buffer
    ///
    /// Only happens on the host.
    ///
    /// This assumes the missing chunks feild is
    /// syncronized with the client. That happens
    /// when the client sends a packet with the status.
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

    /// Copy the status data to a buffer
    ///
    /// Only happens on the client.
    ///
    /// This synchronizes the missing data with the host. 
    /// This should only happen when the client is configuring
    /// the task and the state is [TaskStatus::Configuration]
    pub fn emit_status(&self, buffer: &mut [u8]) {

        buffer[RTNT_DATA_INDEX] = self.id;
        buffer[RTNT_DATA_INDEX+1] = self.total_chunks as u8;
        buffer[RTNT_DATA_INDEX+2] = u8::MAX;

        for i in 0..self.total_chunks {
            buffer[RTNT_DATA_INDEX+3+i] = self.missing_chunks[i] as u8;
        }

    }

    /// Get a reference to the configuration data buffer
    pub fn data(&self) -> &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS] {

        &self.buffer
    
    }
}

/// Node containing an executable, stream, rate, inputs and status
#[cfg_attr(feature = "std", derive(Debug))]
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
    /// Default constructor
    pub fn empty() -> TaskNode {
        TaskNode {

            stream: false,

            rate: 250,
            inputs: [0u8; MAX_TASK_INPUTS],

            data: TaskConfig::default(),
            status: TaskStatus::Standby,
            driver: None,

            task: None,
        }
    }

    /// Create a [TaskNode] from a [TaskExecutable] and some header data.
    ///
    /// This only happens on a host trying to configure
    /// [TaskNode]s from a configuration file.
    pub fn new(stream: bool, rate: u16, driver: TaskDriver, task: TaskExecutable) -> TaskNode {

        let data = task.deconfigure();

        TaskNode {

            stream: stream,

            rate: rate,
            inputs: [0u8; MAX_TASK_INPUTS],

            data: data,
            status: TaskStatus::Configuration,
            driver: Some(driver),

            task: Some(task),
        }
    }

    /// Unnecesary setter, but sets the inputs
    pub fn link(&mut self, link: InputIDBuffer) {

        self.inputs = link;
    
    }

    /// Copy the [TaskNode] data from a buffer to Self
    /// 
    /// If the driver in the header changes the [TaskNode]
    /// will start configuring for that driver immediately. This is only
    /// called when a node recieves an init packet. Only clients
    /// should recieve init packets.
    ///
    /// Also sets the [TaskExecutable] to the (new) driver.
    /// The [TaskExecutable] is reset even if the driver does not
    /// change.
    pub fn init(&mut self, buffer: &[u8]) {
        
        self.stream = buffer[RTNT_HDR_INDEX] > 0;
        self.rate = u16::from_be_bytes([buffer[RTNT_HDR_INDEX+2], buffer[RTNT_HDR_INDEX+3]]);
        self.inputs.copy_from_slice(&buffer[RTNT_HDR_INDEX+4..RTNT_HDR_INDEX+8]);

        let driver = TaskDriver::new(buffer[RTNT_HDR_INDEX+1]);

        // If the [TaskNode] ever receives an init packet with a new driver
        // the node will reset the config data.
        // The config data doesn't need to be reset if the driver is None.
        // The driver can only be set to None when [TaskNode::default()] or [TaskNode::Panic()]
        // is called. In both cases the config data will also be reset.
        //
        // This will generate a new [TaskExecutable] when needed and always set the state to [TaskStatus::Configuration]
        match &mut self.driver {

            None => {

                self.task = Some(TaskExecutable::generate(&driver));
                self.driver = Some(driver);
            
            },
            
            Some(current_driver) => {

                
                if *current_driver != driver {

                    self.task = Some(TaskExecutable::generate(&driver));
                    self.data.clear_chunks();
                    *current_driver = driver;
                
                }
                
            },
        }

        self.status = TaskStatus::Configuration;
    }

    /// Used to share the current header of a task.
    /// This is only useful on a host sending an init packet.
    pub fn dump_header(&self, buffer: &mut [u8]) {
        
        buffer[RTNT_HDR_INDEX] = self.stream as u8;
        buffer[RTNT_HDR_INDEX+1] = match &self.driver { Some(driver) => driver.as_u8(), None => 0, };
        buffer[RTNT_HDR_INDEX+2..RTNT_HDR_INDEX+4].copy_from_slice(&self.rate.to_be_bytes());
        buffer[RTNT_HDR_INDEX+4..RTNT_HDR_INDEX+8].copy_from_slice(&self.inputs);

    }

    /// Collect a packet containing config data.
    /// The data is only consumed if the [TaskNode] has
    /// been put into the [TaskStatus::Configuration] state
    /// by receiving an init packet.
    pub fn collect_chunk(&mut self, buffer: &[u8]) {
        match self.status {
            TaskStatus::Configuration => {

                self.data.collect_chunk(buffer);

            }

            _ => {},
        }
    }

    /// Collect a packet containing config status data.
    /// If the status has missing chunks configuration is
    /// required.
    ///
    /// Returns the number of missing chunks
    pub fn collect_status(&mut self, buffer: &[u8]) -> usize {

        let status = self.data.collect_status(buffer);
        if status != 0 {

            self.status = TaskStatus::Configuration;
        
        }

        status

    }

    /// Resets all task data and enters 
    /// [TaskStatus::Standby] state.
    pub fn kill(&mut self) {

        self.task = None;
        self.driver = None;
        self.data.clear_chunks();
        self.status = TaskStatus::Standby;

    }

    /// Runs the task and returns a buffer
    /// with the nodes header and output data
    pub fn execute(&mut self, inputs: [&[u8]; MAX_TASK_INPUTS], output: &mut [u8]) {

        match &mut self.task {
            None => {},
            Some(task) => task.run(inputs, output),
        }

    }

    /// Configure the task and returns a buffer
    /// with the nodes header and output data
    pub fn configure(&mut self) {

        if self.data.missing_chunks() == 0 {

            self.status = match &mut self.task {

                None => {

                    self.kill();
                    
                    TaskStatus::Standby

                },
                
                Some(task) => {
                
                    match task.configure(self.data.data()) {
                
                        true => TaskStatus::Active,
                
                        false => TaskStatus::Configuration,
                
                    }
                
                },
            
            }

        }

        else {

            self.status = TaskStatus::Configuration;

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

    /// Create a new [TaskManager] with
    /// no tasks.
    pub fn default() -> TaskManager {

        TaskManager {

            context: 0,
            n_nodes: 0,
            nodes: core::array::from_fn(|_| TaskNode::empty()),
            output_buffer: [None; MAX_TASKS],
        
        }
    }

    /// Add a node to the next available index
    /// Does nothing if all nodes are initialized
    pub fn init_node(&mut self, node: TaskNode) {

        if self.n_nodes < MAX_TASKS {

            self.nodes[self.n_nodes] = node;
            self.n_nodes += 1;
        
        } 

    }

    /// Select a packet from the output buffer using
    /// a saved counter.
    pub fn output_by_context(&mut self) -> Option<RIDReport>  {
        let mut ctr = 0;
        let mut output_buffer = [0u8; RID_PACKET_SIZE];

        while ctr < self.n_nodes {
        
            if let Some(publish_buffer) = match self.output_buffer[self.context] {
        
                Some(buffer) => {

                    output_buffer.copy_from_slice(&buffer[..RID_PACKET_SIZE]);
                    Some(output_buffer)
                
                },
        
                None => {

                    ctr += 1;
                    None
                
                }
        
            } {

                self.output_buffer[self.context] = None;

                return Some(publish_buffer);
            
            }
            
            self.context = (self.context + 1) % self.n_nodes;
        
        }

        None
    }

    /// Collect references to the task's input buffers.
    /// (i.e. someother tasks output buffer)
    ///
    /// only returns a ref to the data portion of the output_buffer
    pub fn collect_inputs(&mut self, index: usize) -> [&[u8]; MAX_TASK_INPUTS] {

        core::array::from_fn(|j| {

            let input_index = self.nodes[index].inputs[j] as usize;

            match self.output_buffer[input_index] { 
            
                Some(buffer) => &buffer[RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES], 
            
                None => {
            
                    let buffer = [0u8; RID_PACKET_SIZE];
                    self.output_buffer[input_index] = Some(buffer);

                    &self.output_buffer[input_index].unwrap()

                },
            
            }
        })
    
    }

    /// Collect a packet, identical implementation
    /// for clients and hosts.
    pub fn collect(&mut self, buffer: &RIDReport) -> bool {

        let mut node_select = buffer[RID_MODE_INDEX] as usize;

        match node_select > 0 {
            true => {

                node_select -= 1;

                match PacketType::new(buffer[RID_TOGL_INDEX]) {
                    PacketType::Init => {

                        self.nodes[node_select].init(buffer);
                        false
                    },

                    PacketType::Chunk => {

                        self.nodes[node_select].collect_chunk(buffer);
                        false
                    },

                    PacketType::Status => {

                        self.nodes[node_select].collect_status(buffer);
                        false
                    },

                    PacketType::Data => {

                        self.nodes[node_select].stream

                    },

                    PacketType::Kill => {

                        self.nodes[node_select].kill();
                        false
                    },
                }

            },

            false => false,
        
        }
        
        
    }

    pub fn spin(&mut self) -> Option<RIDReport> {

        for i in 0..MAX_TASKS {
            // the number of nodes includes all tasks
            // with config data in use.
            match self.nodes[i].data.is_init() {
                true => {

                    self.n_nodes = i + 1;

                    match self.nodes[i].status {
                        TaskStatus::Panic => {

                            self.nodes[i].kill();

                        },

                        TaskStatus::Active => {

                            let mut output_buffer = [0u8; RID_PACKET_SIZE];
                            output_buffer[RID_MODE_INDEX] = i as u8 + 1;
                            output_buffer[RID_TOGL_INDEX] = PacketType::Data.as_u8();

                            let inputs = self.collect_inputs(i);
                            self.nodes[i].execute(inputs, &mut output_buffer);

                            self.output_buffer[i] = Some(output_buffer);

                        },

                        TaskStatus::Standby => {},

                        TaskStatus::Configuration => {

                            let mut buffer = [0u8; RID_PACKET_SIZE];
                            buffer[RID_MODE_INDEX] = i as u8 + 1;
                            buffer[RID_TOGL_INDEX] = PacketType::Status.as_u8();

                            self.nodes[i].configure();
                            self.nodes[i].data.emit_status(&mut buffer);

                            self.output_buffer[i] = Some(buffer);

                        },
                        
                    }
                
                },

                false => {},
            }
        }

        self.output_by_context()
    }

    pub fn control_spin(&mut self) -> Option<RIDReport> {

        for i in 0..self.n_nodes {
            // the number of nodes includes all tasks
            // with config data in use.
            match self.nodes[i].status {
                TaskStatus::Panic => {

                    self.nodes[i].kill();

                },

                TaskStatus::Active => {},

                TaskStatus::Standby => {

                    let mut buffer = [0u8; RID_PACKET_SIZE];
                    buffer[RID_MODE_INDEX] = i as u8 + 1;
                    buffer[RID_TOGL_INDEX] = PacketType::Init.as_u8();

                    self.nodes[i].configure();
                    self.nodes[i].dump_header(&mut buffer);

                    self.output_buffer[i] = Some(buffer);
                
                },

                TaskStatus::Configuration => {

                    let mut buffer = [0u8; RID_PACKET_SIZE];
                    buffer[RID_MODE_INDEX] = i as u8 + 1;
                    buffer[RID_TOGL_INDEX] = PacketType::Chunk.as_u8();

                    self.nodes[i].configure();
                    self.nodes[i].data.emit_chunk(&mut buffer);

                    self.output_buffer[i] = Some(buffer);

                },
                
            }
                
        }

        self.output_by_context()
    }

}
