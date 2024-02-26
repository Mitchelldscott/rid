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
	TaskBuffer,
	MAX_TASK_INPUTS,
	MAX_TASK_DATA_BYTES,
	MAX_TASK_CONFIG_CHUNKS,
	rtnt::{
		RTNTask,
		switch::RTSwitch,
		constant::RTConstant,
		task_manager::TaskConfig,
	}
};

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskDriver {
	Switch,
	Constant,
}

impl TaskDriver {
	pub fn new(id: u8) -> TaskDriver {
		match id {
			1 => TaskDriver::Switch,
			_ => TaskDriver::Constant,
		}
	}

	pub fn as_u8(&self) -> u8 {
		match self {
			TaskDriver::Switch => 1,
			TaskDriver::Constant => 2,
		}
	}

	#[cfg(feature = "std")]
	pub fn from_string(s: &str) -> TaskDriver {
		match s {
			"switch" | "Switch" => TaskDriver::Switch,
			"const" | "Constant" | "scalar" => TaskDriver::Constant,
			_ => TaskDriver::Constant,
		}
	}

	#[cfg(feature = "std")]
	pub fn to_string(&self) -> String {
		match self {
			TaskDriver::Switch => "Switch".to_string(),
			TaskDriver::Constant => "Constant".to_string(),
		}
	}
}

#[cfg_attr(feature = "std", derive(Debug))]
pub enum TaskExecutable {
    Switch(RTSwitch),
    Constant(RTConstant),
    // Sinusiod,
    // SquareWave,
    // StateSpace,
    // Polynomial,
}

impl TaskExecutable {

	#[cfg(feature = "std")]
	pub fn load(driver: TaskDriver, data: &str) -> TaskExecutable {

		match driver {
			TaskDriver::Switch => TaskExecutable::Switch(toml::from_str(data).expect("Failed to serialize TaskExecutable::Switch")),
			TaskDriver::Constant => TaskExecutable::Constant(toml::from_str(data).expect("Failed to serialize TaskExecutable::Constant")),
		}
	}

	pub fn generate(driver: &TaskDriver) -> TaskExecutable {
		match driver {
			TaskDriver::Switch => TaskExecutable::Switch(RTSwitch::new()),
			TaskDriver::Constant => TaskExecutable::Constant(RTConstant::new()),
		}
	}

	pub fn run(&mut self, input: [&[u8]; MAX_TASK_INPUTS], output: &mut [u8]) {
		match self {
			TaskExecutable::Switch(task) => task.run(input, output),
			TaskExecutable::Constant(task) => task.run(input, output),
		}
	}

	pub fn configure(&mut self, data: &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS]) -> bool {
		match self {
			TaskExecutable::Switch(task) => task.configure(data),
			TaskExecutable::Constant(task) => task.configure(data),
		}
	}

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