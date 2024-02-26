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
//! # Demonstrate and verify RTN Task implementation
//! The validation criteria for a safe task requires passing
//! more than this test. This test will only demonstrate that
//! the task manager can initialize itself, initialize/execute
//! tasks and share their data.
#![allow(unused_imports)]
use std::fs::read_to_string;

use rid::{
    RID_PACKET_SIZE, MAX_TASK_INPUTS, 
    MAX_TASK_DATA_BYTES, MAX_TASK_CONFIG_CHUNKS, 
    host::{task_builder::*}, 
    rtnt::{
        task_generator::{TaskExecutable, TaskDriver}, 
        task_manager::{TaskStatus, TaskNode, TaskConfig, TaskManager}
    }
};

pub mod rtnt_comms {

    use super::*;

    // pub fn toml_to_tasknode(value: toml::Value::Table) -> TaskNode {


    //     let name = v.0;
    //     let rate = v.1.get("rate").expect("Task {name} is missing a rate");
    //     let driver = v.1.get("driver").expect("Task {name} is missing a driver");

    //     // let data = v.1.get("data").unwrap_or(vec![]);
    //     let stream = v.1.get("stream").unwrap_or(false);
    //     let inputs = v.1.get("inputs").unwrap_or(vec![]);

    //     // data.iter().for_each(|value| {
    //     //     value.to_be_bytes().iter().for_each(|b| {
    //     //         if element < 40 {
    //     //             buffer[chunk][element] = *b;
    //     //             element += 1;
    //     //         }
    //     //         else {
    //     //             element = 0;
    //     //             chunk += 1;
    //     //             assert_le!(chunk, MAX_TASK_CONFIG_CHUNKS, "To many chunks in config file")
    //     //         }
    //     //     })
    //     // });

    //     TaskNode::new(node.stream, node.rate, TaskDriver::new(driver), task_config)

    // }

    pub fn load_file(data: &str) -> TaskManager {

        let toml_data: toml::Value = toml::from_str(read_to_string(data).expect("Failed reading file {data}").as_str()).expect("Filed to convert string to toml");


        let mut raw_experiment = vec![];

        match toml_data {
            toml::Value::Table(value) => { 
                value.into_iter().for_each(|v| {
                                        
                    let name = v.0;
                    let stream = match v.1.get("stream") { Some(toml::Value::Boolean(value)) => *value, _ => false, };
                    let rate = match v.1.get("rate") { Some(toml::Value::Integer(value)) => *value as u16, _ => panic!("Task {name} has a bad rate value"), };
                    let driver = match v.1.get("driver") { Some(toml::Value::String(value)) => value.as_str(), _ => panic!("Task {name} has a bad driver value"), };
                    let inputs = match v.1.get("inputs") { 
                        Some(toml::Value::Array(value)) => {
                            value.into_iter().map(|v| match v {
                                toml::Value::String(value) => value.clone(),
                                _ => panic!("Task {name} has bad inputs"),
                            }).collect()
                        }, 
                        _ => vec![], 
                    };

                    println!("name: {name}\tdriver: {driver}\tinputs: {inputs:?}");

                    let task_exe = TaskExecutable::load(TaskDriver::from_string(driver.clone()), toml::to_string(v.1.get("data").expect("Task is missing data")).expect("Failed to convert task {name}s toml to string").as_str());
                    let node = TaskNode::new(stream, rate, TaskDriver::from_string(driver), task_exe);
                    
                    raw_experiment.push((name, inputs, node));

                });
            },
            _ => panic!("No Table data in file"),
        };

        let mut tm = TaskManager::default();

        let names: Vec<String> = raw_experiment.iter().map(|task| task.0.clone()).collect();

        raw_experiment.sort_by_key(|task| task.1.len()); // This is not actually good, need to do a graph based search ()

        println!("nodes: {names:?}");

        raw_experiment.into_iter().enumerate().for_each(|(i, (name, inputs, mut task))| {

            let mut bytes = [0u8; MAX_TASK_INPUTS];
            
            inputs.iter().enumerate().for_each(|(j, input)| {
                bytes[j] = match names.iter().position(|name| *name == *input) {
                    Some(val) => val as u8,
                    None => panic!("Unable to find input {input} for task {name}"),
                };
            });

            task.status = TaskStatus::Standby;
            task.link(bytes);
            tm.init_node(task);

        });

        tm

    }

    #[test]
    pub fn rtnt_node_dump() {

        let toml_data = "/home/m_dyse/RoPro/rid/examples/data/penguin/nodes.toml";

        let mut tm = TaskManager::default();
        let mut tm_host = load_file(toml_data);

        let mut ctr = 0;

        while ctr < 10 {


            let host_to_client = match tm_host.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent {} {}", tm_host.n_nodes, tm_host.context);
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);


            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent {} {}", tm.n_nodes, tm.context);
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => {}, // println!("publish buffer {publish_buffer:?}"),
                None => {},
            };

            ctr += 1;
        }

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskConfigs {i} did not syncronize");
        }

        ctr = 0;

        while ctr < 3 {


            let host_to_client = match tm_host.panic() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent {}", tm_host.n_nodes);
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);


            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent {}", tm.n_nodes);
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => {}, // println!("publish buffer {publish_buffer:?}"),
                None => {},
            };

            ctr += 1;
        }

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after panic");

        tm_host = load_file(toml_data);


        ctr = 0;

        while ctr < 10 {

            let host_to_client = match tm_host.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);

            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => {}, // println!("publish buffer {publish_buffer:?}"),
                None => {},
            };

            ctr += 1;
        }


        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after reinit");

        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskConfigs did not syncronize after reinit");
        }

    }

    // #[test]
    // pub fn rt_task_spawn() {
    //     let tm = TaskManager::new();
    // }
}