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
//! # Real Time Task Generator
//!
//!   This crate can be included in a firmware build (use the client calls) 
//! or built using the "std" feature.
//!


use crate::{
	rtnt::{
		*,
		switch::RTSwitch, 
		constant::RTConstant,
	}
};

/// Variants that specify which
/// [TaskExecutable] to initialize.
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskDriver {
	/// A simple switch with two inputs and one output (TaskBuffer)
	Switch,
	/// A constant value, no inputs, one output (f32)
	Constant,
}

impl TaskDriver {
	/// Create Self from a u8
	pub fn new(id: u8) -> TaskDriver {
		match id {
			1 => TaskDriver::Switch,
			_ => TaskDriver::Constant,
		}
	}

	/// Convert to a u8
	pub fn as_u8(&self) -> u8 {
		match self {
			TaskDriver::Switch => 1,
			TaskDriver::Constant => 2,
		}
	}

	/// Converts a string to Self
	#[cfg(feature = "std")]
	pub fn from_string(s: &str) -> TaskDriver {
		match s {
			"switch" | "Switch" => TaskDriver::Switch,
			"const" | "Constant" | "scalar" => TaskDriver::Constant,
			_ => TaskDriver::Constant,
		}
	}

	/// Converts to a string
	#[cfg(feature = "std")]
	pub fn to_string(&self) -> String {
		match self {
			TaskDriver::Switch => "Switch".to_string(),
			TaskDriver::Constant => "Constant".to_string(),
		}
	}
}

/// Wrapper object to make task types similar
/// enough to have and array of the type.
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskExecutable {
	/// Simple switch
    Switch(RTSwitch),
    /// Simple constant value
    Constant(RTConstant),
    // Sinusiod,
    // SquareWave,
    // StateSpace,
    // Polynomial,
}

impl TaskExecutable {

	/// Load a [TaskExecutable] from a toml string
	#[cfg(feature = "std")]
	pub fn load(driver: TaskDriver, data: &str) -> TaskExecutable {

		match driver {
			TaskDriver::Switch => TaskExecutable::Switch(toml::from_str(data).expect("Failed to serialize TaskExecutable::Switch")),
			TaskDriver::Constant => TaskExecutable::Constant(toml::from_str(data).expect("Failed to serialize TaskExecutable::Constant")),
		}
	}

	/// Generate a [TaskExecutable] from a driver
	pub fn generate(driver: &TaskDriver) -> TaskExecutable {
		match driver {
			TaskDriver::Switch => TaskExecutable::Switch(RTSwitch::default()),
			TaskDriver::Constant => TaskExecutable::Constant(RTConstant::default()),
		}
	}

	/// Get the number of outputs
	pub fn size(&self) -> usize {
		match self {
			TaskExecutable::Switch(task) => task.size(),
			TaskExecutable::Constant(task) => task.size(),
		}
	}

	/// Call the task and return the output
	pub fn run(&mut self, input: &[f32]) -> TaskData {
		match self {
			TaskExecutable::Switch(task) => task.run(input),
			TaskExecutable::Constant(task) => task.run(input),
		}
	}

	/// Try to configure the tasks private data
	pub fn configure(&mut self, data: &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS]) -> bool {
		match self {
			TaskExecutable::Switch(task) => task.configure(data),
			TaskExecutable::Constant(task) => task.configure(data),
		}
	}

	/// Convert a tasks private data into a [TaskConfig] that can be shared through the [TaskDataCache].
	pub fn deconfigure(&self) -> TaskConfig {
		let mut buffer = [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS];

		match self {
			TaskExecutable::Switch(task) => {
				let total_chunks = task.deconfigure(&mut buffer);
				TaskConfig::new(total_chunks, buffer)
			},
			TaskExecutable::Constant(task) => {
				let total_chunks = task.deconfigure(&mut buffer);
				TaskConfig::new(total_chunks, buffer)
			},
		}
	}
}