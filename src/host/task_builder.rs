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
//! # Real Time Task Builder
//!
//!   This crate can be included in a firmware build (use the client calls) 
//! or built using the "std" feature.
//!
//! Build [TaskExecutable]s from a toml file

use serde::{Serialize, Deserialize};
use std::fs::read_to_string;

use crate::rtnt::{
	task_manager::{TaskManager},
	task_generator::TaskDriver,
};

pub trait ToBeBytes {
    fn to_be_bytes(&self) -> Vec<u8>;
}

impl ToBeBytes for i32 {
    fn to_be_bytes(&self) -> Vec<u8> {
        i32::to_be_bytes(*self).to_vec()
    }
}

impl ToBeBytes for f32 {
    fn to_be_bytes(&self) -> Vec<u8> {
        f32::to_be_bytes(*self).to_vec()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableNode<T> {
    pub name: String,
    pub stream: bool,
    pub rate: u16,
    pub inputs: Vec<String>,
    pub data: Vec<T>,
}

impl<T: Serialize + serde::de::DeserializeOwned + ToBeBytes> SerializableNode<T> {

    pub fn load_file(data: &str) -> TaskManager {

    	let toml_data: toml::Value = toml::from_str(read_to_string(data).expect("Failed reading file {data}").as_str()).unwrap();
        // let mut raw_tasks = vec![];

        match toml_data {
            toml::Value::Table(value) => { 
                value.into_iter().for_each(|v| {
                    
                    let driver = TaskDriver::from_string(v.0.as_str());
                    println!("{driver:?} {:?}", v.1);
                	// let node: SerializableNode<T> = toml::from_str(&toml::to_string(&v.1).expect("Error converting toml data")).expect("Failed converting to SerializedNode");
			        // let mut buffer = [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS];
			        // let mut element = 0;
			        // let mut chunk = 0;

			        // node.data.iter().for_each(|value| {
			        //     value.to_be_bytes().iter().for_each(|b| {
			        //         if element < 40 {
			        //             buffer[chunk][element] = *b;
			        //             element += 1;
			        //         }
			        //         else {
			        //             element = 0;
			        //             chunk += 1;
			        //             assert_le!(chunk, MAX_TASK_CONFIG_CHUNKS, "To many chunks in config file")
			        //         }
			        //     })

			        // });

			        // assert_le!(node.inputs.len(), MAX_TASK_INPUTS, "Node has too many inputs");

			        // let tc = TaskConfig::new(chunk+1, buffer);
                    // raw_tasks.push((v.0, node.inputs, TaskNode::new(node.stream, node.rate, TaskDriver::new(node.driver), tc)));

                });
            },
            _ => panic!("No Table data in file"),
        };

        let mut tm = TaskManager::default();

        // raw_tasks.sort_by_key(|task| task.1.len());

        // let names: Vec<String> = raw_tasks.iter().map(|task| task.0.clone()).collect();

        // println!("nodes: {names:?}");

        // raw_tasks.into_iter().enumerate().for_each(|(i, (name, inputs, mut task))| {

        //     let mut bytes = [0u8; MAX_TASK_INPUTS];
            
        //     inputs.iter().enumerate().for_each(|(j, input)| {
        //         bytes[j] = names.iter().position(|name| *name == *input).expect("Unable to find node associated with input {input}") as u8;
        //     });

        //     task.status = TaskStatus::Standby;
        //     task.link(bytes);
        //     tm.init_node(task);

        // });

        tm

    }

}