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
    RID_PACKET_SIZE, 
    RID_MODE_INDEX, RID_TASK_INDEX,
    rtnt::{
        *,
        task_generator::{TaskExecutable, TaskDriver}, 
        task_manager::{TaskNode, TaskManager}
    }
};

pub mod rtnt_comms {

    use super::*;


    pub fn load_file(data: &str) -> TaskManager {

        let toml_data: toml::Value = toml::from_str(read_to_string(data).expect("Failed reading file {data}").as_str()).expect("Filed to convert string to toml");


        let mut raw_experiment = vec![];

        match toml_data {
            toml::Value::Table(value) => { 
                value.into_iter().for_each(|v| {
                                        
                    let name = v.0;
                    let stream = match v.1.get("stream") { Some(toml::Value::Boolean(value)) => *value as u8, _ => 0, };
                    let rate = match v.1.get("rate") { Some(toml::Value::Integer(value)) => *value as u16, _ => panic!("Task {name} has a bad rate value"), };
                    let outs = match v.1.get("n_outputs") { Some(toml::Value::Integer(value)) => *value as u8, _ => panic!("Task {name} has a bad number of output values"), };
                    let driver = match v.1.get("driver") { Some(toml::Value::String(value)) => value.as_str(), _ => panic!("Task {name} has a bad driver value"), };
                    
                    let mut inputs = vec![];
                    match v.1.get("inputs") { 
                        Some(toml::Value::Array(value)) => {
                            value.into_iter().for_each(|v| match v {
                                toml::Value::String(value) => {
                                    let mut split = value.split('.');
                                    match split.next() {
                                        Some(name) => {
                                            match split.next() {
                                                Some(index) => inputs.push((name.to_string(), index.parse::<u8>().expect("Failed parsing input {name} index"))),
                                                None => inputs.push((name.to_string(), 0)),
                                            }
                                        }
                                        None => panic!("Split should never return None... see the crate docs or something"),
                                    }
                                },
                                _ => panic!("Task {name} has bad input values"),
                            });
                        }, 
                        _ => {}, 
                    };

                    println!("name: {name}\tdriver: {driver}\tinputs: {inputs:?}");

                    let task_exe = TaskExecutable::load(TaskDriver::from_string(driver), toml::to_string(v.1.get("data").expect("Task is missing data")).expect("Failed to convert task {name}s toml to string").as_str());
                    let node = TaskNode::new(stream, rate, inputs.len() as u8, outs, TaskDriver::from_string(driver), task_exe);
                    
                    raw_experiment.push((name, inputs, node));

                });
            },
            _ => panic!("No Table data in file"),
        };

        let mut tm = TaskManager::default();

        let names: Vec<String> = raw_experiment.iter().map(|task| task.0.clone()).collect();

        raw_experiment.sort_by_key(|task| task.1.len()); // This is not actually good, need to do a graph based search ()

        raw_experiment.into_iter().enumerate().for_each(|(_, (name, inputs, mut task))| {

            let mut bytes = [[0u8; 2]; MAX_TASK_INPUTS];
            
            inputs.iter().enumerate().for_each(|(j, input)| {
                match names.iter().position(|name| *name == input.0) {
                    Some(val) => {
                        bytes[j][0] = val as u8;
                        bytes[j][1] = input.1;
                    },
                    None => panic!("Unable to find input {input:?} for task {name}"),
                };
            });

            task.link(bytes);
            tm.init_node(task);

        });

        tm

    }


    pub fn spin_local(n: usize, tm: &mut TaskManager, tm_host: &mut TaskManager) {
        let mut ctr = 0;

        while ctr < n {

            println!("Total Nodes: host {}, client {}", tm_host.n_nodes, tm.n_nodes);

            let host_to_client = match tm_host.control_spin() {
                Some(buffer) => {
                    // println!("\t[host -> client] node: {} mode: {:?}", buffer[RID_TASK_INDEX] as i8 - 1, PacketType::new(buffer[RID_MODE_INDEX]));
                    buffer
                },
                None => {
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);


            let client_to_host = match tm.spin() {
                Some(buffer) => {
                    // println!("\t[client -> host] node: {} mode: {:?}", buffer[RID_TASK_INDEX] as i8 - 1, PacketType::new(buffer[RID_MODE_INDEX]));
                    buffer
                },
                None => {
                    [0u8; RID_PACKET_SIZE]
                },
            };

            if tm_host.collect(&client_to_host) {


                let node = client_to_host[RID_TASK_INDEX] - 1;
                let mut data = [0f32; MAX_TASK_DATA_FLOATS];

                for i in 0..MAX_TASK_DATA_FLOATS {
                    data[i] = f32::from_be_bytes([client_to_host[RTNT_DATA_INDEX+(4*i)],client_to_host[RTNT_DATA_INDEX+(4*i)+1],client_to_host[RTNT_DATA_INDEX+(4*i)+2],client_to_host[RTNT_DATA_INDEX+(4*i)+3]]);
                }

                println!("\tData from node[{}] = {:?}", node, (0..tm_host.nodes[node as usize].n_outputs).map(|i| data[i as usize]).collect::<Vec<f32>>());

            }

            ctr += 1;
        }

    }

    #[test]
    pub fn rtnt_load_panic_load_load_panic() {

        let toml_data = "examples/data/penguin/nodes.toml";
        let toml_data_alt = "examples/data/penguin/nodes_alt.toml";

        let mut tm = TaskManager::default();
        let mut tm_host = load_file(toml_data);

        println!("===== Load 1 =====");
        spin_local(10, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize");
            }
        }

        println!("===== Panic 1 =====");
        tm_host.panic_all();
        spin_local(1, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after panic");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize after panic");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize after panic");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize after panic");
            }
        }

        tm_host = load_file(toml_data);

        println!("===== Load 1 =====");
        spin_local(10, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after first reinit");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize after first reinit");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize after first reinit");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize after first reinit");
            }
        }

        println!("===== Load 3 =====");
        tm_host = load_file(toml_data_alt);

        spin_local(10, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after second reinit");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize after second reinit");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize after second reinit");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize after second reinit");
            }
        }

        println!("===== Panic 2 =====");
        tm_host.panic_all();
        spin_local(1, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize after second panic");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize after second panic");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize after second panic");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize after second panic");
            }
        }

    }

    #[test]
    pub fn rtnt_load_reconfigure() {

        let toml_data_alt = "examples/data/penguin/nodes_alt.toml";

        let mut tm = TaskManager::default();
        let mut tm_host = load_file(toml_data_alt);

        println!("===== Load 1 =====");
        spin_local(10, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize");
            }
        }


        for i in 0..tm_host.n_nodes {
            let task = TaskExecutable::Constant(rid::rtnt::constant::RTConstant::new(i as f32));
            let driver = TaskDriver::Constant;
            let stream = 1;
            let rate = 100;
            let ins = 0;
            let outs = 1;

            tm_host.nodes[i].modify(stream, rate, ins, outs, driver, task);
        }

        spin_local(11, &mut tm, &mut tm_host);

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].status, tm.nodes[i].status, "TaskStatus {i} did not syncronize");
            assert_eq!(tm_host.nodes[i].config_cache, tm.nodes[i].config_cache, "TaskConfigs {i} did not syncronize");
            if tm_host.nodes[i].stream > 0 {
                assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskData {i} did not syncronize");
            }
        }
        
    }

}




