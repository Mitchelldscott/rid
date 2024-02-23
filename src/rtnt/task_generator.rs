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
	MAX_TASK_CONFIG_CHUNKS,
	rtnt::{
		default::RTNTask,
		default::RTDefault,
		switch::RTSwitch,
	}
};


pub enum TaskDriver {
	Default,
	Switch,
}

impl TaskDriver {
	pub fn new(id: u8) -> TaskDriver {
		match id {
			0 => TaskDriver::Switch,
			_ => TaskDriver::Default,			
		}
	}
}


pub enum TaskExecutable {
    Default(RTDefault),
    Switch(RTSwitch),
    // Sinusiod,
    // SquareWave,   
    // StateSpace,
    // Polynomial,
}

impl TaskExecutable {
	pub fn generate(driver: TaskDriver) -> TaskExecutable {
		match driver {
			TaskDriver::Default => TaskExecutable::Default(RTDefault::new()),
			TaskDriver::Switch => TaskExecutable::Switch(RTSwitch::new()),
		}
	}

	pub fn configure(&mut self, data: &[TaskBuffer; MAX_TASK_CONFIG_CHUNKS]) -> bool {
		match self {
			TaskExecutable::Default(task) => task.configure(data),
			TaskExecutable::Switch(task) => task.configure(data),
		}
	}
}