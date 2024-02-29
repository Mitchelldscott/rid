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

use crate::{RIDReport, RID_PACKET_SIZE, RID_TASK_INDEX, RID_MODE_INDEX, RID_PTP_RESERVED_BYTES};

/// Maximum number of tasks user can spawn
pub const MAX_TASKS: usize = 32;
/// Maximum number of bytes a task can use in its name
pub const MAX_TASK_INPUTS: usize = 16;
/// Maximum number of configuration chunks each task can use
pub const MAX_TASK_CONFIG_CHUNKS: usize = 32;

/// Cnostant
/// RTNT Header index
pub const RTNT_HDR_INDEX: usize = 0;
/// RTNT Header length
pub const RTNT_HDR_LENGTH: usize = 8;
/// RTNT Data start
pub const RTNT_DATA_INDEX: usize = RTNT_HDR_LENGTH + RTNT_HDR_INDEX;
/// Maximum number of bytes in a Tasks output buffer and configuration chunk
pub const MAX_TASK_DATA_BYTES: usize = RID_PACKET_SIZE - RTNT_DATA_INDEX - RID_PTP_RESERVED_BYTES;
/// Maximum nuber of floats in a tasks output (use f32, not worried about precision rn)
pub const MAX_TASK_DATA_FLOATS: usize = MAX_TASK_DATA_BYTES / 4;

/// alias for readability
pub type TaskHeader = [u8; RTNT_HDR_LENGTH];

/// alias for readability
pub type TaskBuffer = [u8; MAX_TASK_DATA_BYTES];

/// alias for readability
pub type InputIDBuffer = [[u8; 2]; MAX_TASK_INPUTS];

/// alias for readability
pub type TaskData = [f32; MAX_TASK_DATA_FLOATS];

/// Trait for RTNTasks
pub trait RTNTask {

    /// Create a new exectuable
    fn default() -> Self;

    /// The number of values in the output
    fn size(&self) -> usize;

    /// Once the configuration chunks have all arrived pass the buffer to this.
    /// Given all chunks this function will initialize the [RTNTask]s members
    fn configure(&mut self, data: &[TaskBuffer]) -> bool;
    /// Convert the tasks members into a TaskConfig
    fn deconfigure(&self, data: &mut [TaskBuffer]) -> usize;

    /// Run the executable 
    fn run(&mut self, input: &[f32]) -> TaskData;

}

/// Specifies the state a Task is in
/// and the action required by the [crate::rtnt::task_manager::TaskManager]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskStatus {
    /// The task is panicing due to a runtime/configuration error
    Panic,
    /// The task is executing normally
    Active,
    /// No task or driver is initialized
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

/// Describes the contents of an [RIDReport].
/// These variants can control remote instances
/// of the a task.
///
/// Example
/// Client has an unconfigured task, so it sends a status
/// to the host. Now the host can share the proper chunks.
///
/// Example
/// Host loads [crate::rtnt::task_manager::TaskNode]s from a file, the tasks begin in [TaskStatus::Standby].
/// This leads to the host sending PacketType::Init, until it recieves a status
/// and enters the [TaskStatus::Configuration] state.
///
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum PacketType {
    /// init data for a [crate::rtnt::task_manager::TaskNode]
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

    /// Create a buffer with the given number of chunks and buffer
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
    pub fn clear_missing(&mut self) {
        
        self.missing_chunks = [false; MAX_TASK_CONFIG_CHUNKS];

    }

    /// Check if the buffer is being used
    pub fn init(&mut self) {

        self.total_chunks = 1;
        self.reset_chunks();
    
    }

    /// Check if the buffer is being used
    pub fn is_init(&self) -> bool {

        self.total_chunks > 0
    
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
    ///
    /// This is only valid when total_chunks
    /// is set to the correct value.
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
    ///
    /// This is only valid when total_chunks
    /// is set to the correct value.
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
    pub fn collect_chunk(&mut self, header: &[u8], buffer: &[u8]) {

        let id = header[2];
        let chunk_num = header[3] as usize;

        self.total_chunks = header[4] as usize;

        match self.id == id {
            true => {

                self.new_chunk(chunk_num, &buffer[..MAX_TASK_DATA_BYTES]);

            },
            _ => {

                self.id = id;
                self.reset_chunks();
                self.new_chunk(chunk_num, &buffer[..MAX_TASK_DATA_BYTES]);

            },
        }
    }

    /// Copy the chunk status from the buffer.
    ///
    /// This will only happen on hosts trying to
    /// configure a set of tasks.
    pub fn collect_status(&mut self, buffer: &[u8]) -> usize {

        for i in 0..self.total_chunks {
            self.missing_chunks[i] = buffer[i] != 0;
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
    pub fn emit_chunk(&self) -> (TaskHeader, TaskBuffer) {

        let mut header = [0u8; RTNT_HDR_LENGTH];
        let mut buffer = [0u8; MAX_TASK_DATA_BYTES];

        let chunk_num = match self.first_missing() {
            Some(chunk_num) => {
                buffer[..MAX_TASK_DATA_BYTES].copy_from_slice(&self.buffer[chunk_num]);

                chunk_num as u8
            }
            _ => 0,
        };

        header[2] = self.id;
        header[3] = chunk_num;
        header[4] = self.total_chunks as u8;

        (
            header,
            buffer
        )
    }

    /// Copy the status data to a buffer
    ///
    /// Only happens on the client.
    ///
    /// This synchronizes the missing data with the host. 
    /// This should only happen when the client is configuring
    /// the task and the state is [TaskStatus::Configuration]
    pub fn emit_status(&self) -> TaskBuffer {

        let mut buffer = [0u8; MAX_TASK_DATA_BYTES];

        for i in 0..self.total_chunks {
            buffer[i] = self.missing_chunks[i] as u8;
        }

        buffer

    }

    /// Get a reference to the configuration data buffer
    pub fn data(&self) -> &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS] {

        &self.buffer
    
    }
}

/// A buffer for all system outputs
/// 
/// Has a context that will set which task to stream.
/// Can make this more complex if the field becomes a priority based queue, 
/// (i.e. stream = 0: no instances, stream = n: n uniformly distributed instances in queue)
/// the queue can be built/restructured as nodes are created, each element of the queue
/// will be the index of the desired task to stream. [crate::rtnt::task_manager::TaskNode] does not need to know if nodes stream.
pub struct TaskDataCache {
    /// current task to publish
    context: usize,
    /// Empty buffer for empty inputs
    empty: TaskBuffer,
    /// buffer of task data
    buffer: [RIDReport; MAX_TASKS],
}

impl TaskDataCache {

    /// Construct an empty cache
    pub fn default() -> TaskDataCache {
        TaskDataCache {
            context: 0,
            empty: [0u8; MAX_TASK_DATA_BYTES],
            buffer: [[0u8; RID_PACKET_SIZE]; MAX_TASKS],
        }
    }

    /// Use the context to find the first task with unpublished output (buffer[RID_TASK_INDEX] > 0).
    /// Copy the output to a new buffer and clear the mode index.
    pub fn publish(&mut self, context_wrap: usize) -> Option<RIDReport> {
        let mut ctr = 0;
        let mut publish_buffer = [0u8; RID_PACKET_SIZE];

        while ctr < context_wrap {
        
            if let Some(publish_buffer) = match self.buffer[self.context][RID_TASK_INDEX] > 0 {
        
                true => {

                    publish_buffer.copy_from_slice(&self.buffer[self.context][..RID_PACKET_SIZE]);
                    Some(publish_buffer)
                
                },
        
                false => {

                    ctr += 1;
                    None
                
                }
        
            } {

                self.buffer[self.context][RID_TASK_INDEX] = 0; // No data available
                self.context = (self.context + 1) % context_wrap;

                return Some(publish_buffer);
            
            }
            
            self.context = (self.context + 1) % context_wrap;
        
        }

        None
    }

    /// If the buffer has an unpublished [PacketType::Status]
    /// Used to prevent the client from overwriting status packets with
    /// data packets (Don't stream data without telling the host the task is ready).
    pub fn status_waiting(&self, index: usize) -> bool {

        self.buffer[index][RID_TASK_INDEX] != 0 && self.buffer[index][RID_MODE_INDEX] == PacketType::Status.as_u8()

    }

    /// Mutable reference to a tasks buffer
    pub fn task_buffer(&mut self, index: usize) -> &[u8] {
        
        if index >= MAX_TASKS { panic!("Invalid index to TaskDataCache {index}"); }
        
        &self.buffer[index]
    
    }

    /// Publish a new status and TaskBuffer to a tasks cache. Will
    /// always overwrite unpublished packets. If that is not desired
    /// The buffer should be checked before hand.
    pub fn new_output(&mut self, index: usize, status: PacketType, buffer: TaskBuffer) {
        
        if index >= MAX_TASKS { panic!("Invalid index to TaskDataCache {index}"); }


        self.buffer[index][RID_TASK_INDEX] = index as u8 + 1;
        self.buffer[index][RID_MODE_INDEX] = status.as_u8();

        self.buffer[index][RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES].copy_from_slice(&buffer[..MAX_TASK_DATA_BYTES]);

    }

    /// Write a new mode and TaskBuffer to a task's cache. This packet can always be overwritten and
    /// will not stream to remote instances. 
    pub fn new_nonstreaming_output(&mut self, index: usize, mode: PacketType, buffer: TaskBuffer) {
        
        if index >= MAX_TASKS { panic!("Invalid index to TaskDataCache {index}"); }

        self.buffer[index][RID_TASK_INDEX] = 0;
        self.buffer[index][RID_MODE_INDEX] = mode.as_u8();
        
        self.buffer[index][RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES].copy_from_slice(&buffer[..MAX_TASK_DATA_BYTES]);
    
    }

    /// Publish a new [PacketType], [TaskHeader] and [TaskBuffer].
    /// This function will overwrite any existing data.
    pub fn new_output_with_header(&mut self, index: usize, mode: PacketType, mut header: TaskHeader, buffer: TaskBuffer) {
        
        if index >= MAX_TASKS { panic!("Invalid index to TaskDataCache {index}"); }

        header[RID_TASK_INDEX] = index as u8 + 1;
        header[RID_MODE_INDEX] = mode.as_u8();
        
        self.buffer[index][RTNT_HDR_INDEX..RTNT_DATA_INDEX].copy_from_slice(&header);
        self.buffer[index][RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES].copy_from_slice(&buffer[..MAX_TASK_DATA_BYTES]);
    
    }

    /// Get a reference to all the requested task buffers.
    /// Not a mutable reference and does not make any garuantees
    /// about the data behind the reference.
    pub fn task_input_buffer(&self, inputs: &[u8; MAX_TASK_INPUTS]) -> [&[u8]; MAX_TASK_INPUTS] {
        
        core::array::from_fn(|i| {

            match inputs[i as usize] < MAX_TASKS as u8 { 
            
                true => &self.buffer[inputs[i as usize] as usize][RTNT_DATA_INDEX..RTNT_DATA_INDEX+MAX_TASK_DATA_BYTES],
            
                false => &self.empty,
            
            }
            
        })
    
    }
}

pub mod task_generator;
pub mod task_manager;

pub mod switch;
pub mod constant;
