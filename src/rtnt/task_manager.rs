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
    RID_TASK_INDEX, RID_MODE_INDEX,
    RIDReport, 
    rtnt::{*, task_generator::*},
};

/// Node containing an executable, stream, rate, inputs and status
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TaskNode {
    /// Rate this task will execute at
    pub rate: u16,
    /// Speceifies if the data should be streamed
    pub stream: u8,
    /// number of input values
    pub n_inputs: u8,
    /// number of output values
    pub n_outputs: u8,
    /// Input Tasks
    pub inputs: InputIDBuffer,
    /// Output data
    pub data: TaskData,

    /// Status of the task, enables and disables running the task
    pub status: TaskStatus,
    /// Buffer containing configuration data for the task
    pub config_cache: TaskConfig,

    /// The driver assigned to this node
    pub driver: Option<TaskDriver>,
    /// Optional Executable, there is always the max number of [TaskNode]s
    /// but not all will have [TaskExecutable]s
    pub task: Option<TaskExecutable>,

}

impl TaskNode {
    /// Default constructor
    pub fn empty() -> TaskNode {
        TaskNode {

            rate: 250,
            stream: 0,
            n_inputs: 0,
            n_outputs: 0,
            inputs: [[0u8; 2]; MAX_TASK_INPUTS],
            data: [0.0f32; MAX_TASK_DATA_FLOATS],

            status: TaskStatus::Standby,
            config_cache: TaskConfig::default(),

            driver: None,
            task: None,
        }
    }

    /// Create a [TaskNode] from a [TaskExecutable] and some header data.
    ///
    /// This only happens on a host trying to configure
    /// [TaskNode]s from a file. On clients [TaskNode]s are built by 
    /// collecting chunk packets from the host.
    pub fn new(stream: u8, rate: u16, inputs: u8, outputs: u8, driver: TaskDriver, task: TaskExecutable) -> TaskNode {

        let cache = task.deconfigure();

        TaskNode {

            rate: rate,
            stream: stream,
            n_inputs: inputs,
            n_outputs: outputs,
            inputs: [[0u8; 2]; MAX_TASK_INPUTS],
            data: [0.0f32; MAX_TASK_DATA_FLOATS],

            status: TaskStatus::Standby,
            config_cache: cache,

            driver: Some(driver),
            task: Some(task),
        }
    }

    /// Modify a [TaskNode] from a [TaskExecutable] and some header data.
    ///
    /// This only happens on a host trying to configure
    /// [TaskNode]s from a file. On clients [TaskNode]s are built by 
    /// collecting chunk packets from the host.
    pub fn modify(&mut self, stream: u8, rate: u16, inputs: u8, outputs: u8, driver: TaskDriver, task: TaskExecutable) {

        let cache = task.deconfigure();

        self.rate = rate;
        self.stream = stream;
        self.n_inputs = inputs;
        self.n_outputs = outputs;
        self.inputs = [[0u8; 2]; MAX_TASK_INPUTS];
        self.data = [0.0f32; MAX_TASK_DATA_FLOATS];

        self.status = TaskStatus::Standby;
        self.config_cache = cache;

        self.driver = Some(driver);
        self.task = Some(task);

    }

    /// Unnecesary setter, but sets the inputs
    pub fn link(&mut self, link: InputIDBuffer) {

        self.inputs = link;
    
    }

    /// Copy the [TaskNode] header data from a buffer to Self
    /// 
    /// If the driver in the header changes the [TaskNode]
    /// will start configuring for that driver immediately. 
    /// Called when a client recieves an init packet. Only clients
    /// should recieve init packets.
    ///
    /// Also sets the [TaskExecutable] to the new driver (if it is a new driver).
    pub fn init(&mut self, header: &[u8], data: &[u8]) {
        
        let driver = TaskDriver::new(header[5]);

        self.stream = header[2];
        self.rate = u16::from_be_bytes([header[3], header[4]]);
        self.n_inputs = header[6];
        self.n_outputs = header[7];

        for i in 0..self.n_inputs as usize {

            self.inputs[i][0] = data[2*i];
            self.inputs[i][1] = data[(2*i)+1];
        
        }

        self.config_cache.init();

        // If the [TaskNode] ever receives an init packet
        // the node will reset the config data. The config data doesn't 
        // need to be reset if the driver is None or matches the one in the packet.
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
                    self.config_cache.init();
                    *current_driver = driver;
                
                }
                
            },
        }

        self.status = TaskStatus::Configuration;
    }

    /// Used to share the current header of a task.
    /// This is only useful on a host sending an init packet.
    pub fn header(&self) -> (TaskHeader, TaskBuffer) {
        
        let mut header = [0u8; RTNT_HDR_LENGTH];
        let mut data = [0u8; MAX_TASK_DATA_BYTES];

        header[2] = self.stream;
        header[3..5].copy_from_slice(&self.rate.to_be_bytes());
        header[5] = match &self.driver { Some(driver) => driver.as_u8(), None => 0, };
        header[6] = self.n_inputs;
        header[7] = self.n_outputs;

        for i in 0..self.n_inputs as usize {
            data[2*i] = self.inputs[i][0];
            data[(2*i)+1] = self.inputs[i][1];
        }
        // data[..MAX_TASK_INPUTS].copy_from_slice(&self.inputs);

        (header, data)

    }

    /// Collect a packet containing config data.
    /// The data is only consumed if the [TaskNode] is
    /// already in the [TaskStatus::Configuration] state.
    pub fn collect_chunk(&mut self, header: &[u8], data: &[u8]) {
        match self.status {
            TaskStatus::Configuration => {

                self.config_cache.collect_chunk(header, data);

            }

            _ => {},
        }
    }

    /// Resets all task data and enters 
    /// [TaskStatus::Standby] state.
    pub fn kill(&mut self) {

        self.task = None;
        self.driver = None;
        self.config_cache.clear_chunks();
        self.status = TaskStatus::Standby;

    }

    /// Configure the task. Activates the task if all chunks
    /// have been collected. If the [TaskExecutable] is not
    /// initialized there was an error initializing and the
    /// node reached this state in error, revert to standby.
    pub fn configure(&mut self) {

        if self.config_cache.missing_chunks() == 0 {

            self.status = match &mut self.task {

                None => {

                    self.kill();

                    // println!("No task!");
                    
                    TaskStatus::Standby

                },
                
                Some(task) => {
                
                    match task.configure(self.config_cache.data()) {
                
                        true => TaskStatus::Active,
                
                        false => TaskStatus::Panic,
                
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

    /// the number of active nodes
    pub n_nodes: usize,
    /// list of nodes
    pub nodes: [TaskNode; MAX_TASKS],
    /// buffer containing each tasks output data
    pub data_cache: TaskDataCache,

}


impl TaskManager {

    /// Create a new [TaskManager] with
    /// no tasks.
    pub fn default() -> TaskManager {

        TaskManager {

            n_nodes: 0,
            nodes: core::array::from_fn(|_| TaskNode::empty()),
            data_cache: TaskDataCache::default(),
        
        }
    }

    /// Add a node to the next available index
    /// Does nothing if all indices are initialized
    pub fn init_node(&mut self, node: TaskNode) {

        if self.n_nodes < MAX_TASKS {

            self.nodes[self.n_nodes] = node;
            self.n_nodes += 1;
        
        } 

    }

    /// Panic all nodes
    /// 
    /// Causes the client to kill all nodes.
    pub fn panic_all(&mut self) {

        for i in 0..self.n_nodes {

            self.nodes[i].status = TaskStatus::Panic;
        
        } 

    }

    /// Collect a packet. This is the [TaskManager]s interface
    /// to a remote counter part. 
    ///
    /// Identical implementation for clients and hosts. 
    /// The [PacketType], embedded as as a u8 at 
    /// [RID_MODE_INDEX], determines how the node
    /// should handle the data. Any node receiving a panic
    /// will call [TaskNode::kill()] and deinitialize.
    ///
    /// The [TaskNode] packet collector functions will assert 
    /// the node is in the correct state before handling any data.
    pub fn collect(&mut self, buffer: &RIDReport) -> bool {

        let mut node_select = buffer[RID_TASK_INDEX] as usize;

        match node_select > 0 {
            true => {

                node_select -= 1;

                match PacketType::new(buffer[RID_MODE_INDEX]) {
                    PacketType::Init => {

                        self.nodes[node_select].init(&buffer[RTNT_HDR_INDEX..RTNT_DATA_INDEX], &buffer[RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES]);
                        false

                    },

                    PacketType::Chunk => {

                        self.nodes[node_select].collect_chunk(&buffer[RTNT_HDR_INDEX..RTNT_DATA_INDEX], &buffer[RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES]);
                        false

                    },

                    PacketType::Status => {

                        self.nodes[node_select].config_cache.collect_status(&buffer[RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES]);
                        self.nodes[node_select].status = match self.nodes[node_select].config_cache.missing_chunks() > 0 { true => TaskStatus::Configuration, false => TaskStatus::Active, };
                        // println!("Status reply {node_select} {}", self.nodes[node_select].data.missing_chunks());
                        false

                    },

                    PacketType::Data => {

                        self.nodes[node_select].status = TaskStatus::Active;
                        let bytes = &buffer[RTNT_DATA_INDEX..MAX_TASK_DATA_BYTES];

                        for i in 0..self.nodes[node_select].n_outputs as usize {
                            
                            self.nodes[node_select].data[i] = f32::from_be_bytes([bytes[i*4], bytes[(i*4)+1], bytes[(i*4)+2], bytes[(i*4)+3]]);

                        }
                        self.nodes[node_select].stream > 0

                    },

                    PacketType::Kill => {

                        for i in 0..self.n_nodes {

                            self.nodes[i].status = TaskStatus::Panic;
                        
                        }

                        false
                    },
                }

            },

            false => false,
        
        }
        
        
    }

    /// Produce a packet and try configuring or exectuing each task. This is the output
    /// interface for a host counter part. The packets this function produces shoud be delivered
    /// to a host instance. This function should be paired with [TaskManager::collect()] in an
    /// embedded system.
    pub fn spin(&mut self) -> Option<RIDReport> {

        for i in 0..MAX_TASKS {
            // the number of nodes includes all tasks
            // with config data in use.
            match self.nodes[i].config_cache.is_init() {
                true => {

                    self.n_nodes = i + 1;

                    match self.nodes[i].status {
                        TaskStatus::Panic => {

                            for i in 0..MAX_TASKS {
                                self.nodes[i].kill();
                            }

                            self.n_nodes = 0;

                        },

                        TaskStatus::Active => {
                            
                            if !self.data_cache.status_waiting(i) {

                                // let inputs = self.data_cache.task_input_buffer(&self.nodes[i].inputs);
                                let mut inputs = [0.0f32; MAX_TASK_DATA_FLOATS];

                                for j in 0..self.nodes[i].n_inputs as usize {
                                    
                                    let id = self.nodes[i].inputs[j][0] as usize;
                                    let index = self.nodes[i].inputs[j][1] as usize;
                                    inputs[i] = self.nodes[id].data[index];

                                }

                                if let Some(output) = match &mut self.nodes[i].task {
                                    None => None,
                                    Some(task) => Some(task.run(&inputs[..MAX_TASK_DATA_FLOATS])),
                                }
                                {

                                    self.nodes[i].data = output;

                                    if self.nodes[i].stream > 0 {

                                        let mut buffer = [0u8; MAX_TASK_DATA_BYTES];
                                        
                                        for j in 0..self.nodes[i].n_outputs as usize {
                                            buffer[(4*j)..(4*j)+4].copy_from_slice(&self.nodes[i].data[j].to_be_bytes());
                                        }

                                        self.data_cache.new_output(i, PacketType::Data, buffer);
                                    }
                                }

                            }

                        },

                        TaskStatus::Standby => {},

                        TaskStatus::Configuration => {

                            self.nodes[i].configure();

                            self.data_cache.new_output(i, PacketType::Status, self.nodes[i].config_cache.emit_status());
                            self.data_cache.context = i;

                        },
                        
                    }
                
                },

                false => {},
            }
        }

        self.data_cache.publish(self.n_nodes)
    }

    /// Produce configuration packets for loaded tasks. This is the output
    /// interface for a client counter part. The packets this function produces shoud be delivered
    /// to a client instance. This function should be paired with [TaskManager::collect()] on a host machine.
    pub fn control_spin(&mut self) -> Option<RIDReport> {

        for i in 0..self.n_nodes {

            // the number of nodes includes all tasks
            // with config data in use.
            match self.nodes[i].status {
                TaskStatus::Panic => {

                    for i in 0..self.n_nodes {
                        self.nodes[i].kill();
                    }

                    self.data_cache.new_output(0, PacketType::Kill, [0u8; MAX_TASK_DATA_BYTES]);
                    self.data_cache.context = 0;

                    self.n_nodes = 0;
                    return self.data_cache.publish(1);

                },

                TaskStatus::Active => {},

                TaskStatus::Standby => {

                    let (header, data) = self.nodes[i].header();
                    self.data_cache.new_output_with_header(i, PacketType::Init, header, data);

                },

                TaskStatus::Configuration => {

                    let (header, data) = self.nodes[i].config_cache.emit_chunk();
                    self.data_cache.new_output_with_header(i, PacketType::Chunk, header, data);

                },
                
            }
                
        }

        self.data_cache.publish(self.n_nodes)
    }

}
