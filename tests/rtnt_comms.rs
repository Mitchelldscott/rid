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

use rid::{RID_PACKET_SIZE, MAX_TASK_INPUTS, MAX_TASK_DATA_BYTES, MAX_TASK_CONFIG_CHUNKS, rtnt::{task_generator::TaskDriver, task_manager::{TaskStatus, TaskNode, TaskConfig, TaskManager}}};

pub mod rtnt_comms {

    use super::*;

    use serde::{Serialize, Deserialize};
    use more_asserts::assert_le;

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
        pub stream: bool,
        pub rate: u16,
        pub driver: u8,
        pub inputs: Vec<String>,
        pub data: Vec<T>
    }

    impl<T: Serialize + serde::de::DeserializeOwned + ToBeBytes> SerializableNode<T> {

        pub fn load(data: &str) -> (Vec<String>, TaskNode) {

            let node: SerializableNode<T> = toml::from_str(data).unwrap();
            let mut buffer = [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS];
            let mut element = 0;
            let mut chunk = 0;

            node.data.iter().for_each(|value| {
                value.to_be_bytes().iter().for_each(|b| {
                    if element < 40 {
                        buffer[chunk][element] = *b;
                        element += 1;
                    }
                    else {
                        element = 0;
                        chunk += 1;
                        assert_le!(chunk, MAX_TASK_CONFIG_CHUNKS, "To many chunks in config file")
                    }
                })

            });

            assert_le!(node.inputs.len(), MAX_TASK_INPUTS, "Node has too many inputs");

            let tc = TaskConfig::new(chunk+1, buffer);
            println!("Chunks {}", tc.chunks());
            println!("Missing Chunks {}", tc.missing_chunks());
            (node.inputs, TaskNode::new(node.stream, node.rate, [0u8; MAX_TASK_INPUTS], TaskDriver::new(node.driver), tc))

        }

    }

    #[test]
    pub fn rtnt_node_dump() {
        let mut tm = TaskManager::default();
        let mut tm_host = TaskManager::default();
        // let toml = toml::to_string(&tm.nodes[0]).unwrap();
        // println!("{toml}");
        // let tm_host = TaskManager::new(toml::from_file())
        let toml_data = r#"
            [switch1]
            stream = false
            rate = 100
            inputs = ["switch2"]
            driver = 1
            data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0]
            [switch2]
            stream = true
            rate = 100
            inputs = []
            driver = 1
            data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0]
        "#;

        let data: toml::Value = toml::from_str(toml_data).unwrap();
        let mut raw_tasks = vec![];

        match data {
            toml::Value::Table(value) => { 
                value.into_iter().for_each(|v| {
                    let (inputs, task) = SerializableNode::<f32>::load(&toml::to_string(&v.1).expect("Error converting toml data"));
                    raw_tasks.push((v.0, inputs, task))
                });
            },
            _ => panic!("No Table data in file"),
        };

        raw_tasks.sort_by_key(|task| task.1.len());

        let names: Vec<String> = raw_tasks.iter().map(|task| task.0.clone()).collect();
        // let inputs: Vec<Vec<String>> = raw_tasks.iter().map(|task| task.1.clone()).collect();
        // let mut tasks: Vec<TaskNode> = raw_tasks.into_iter().map(|task| task.2).collect();

        println!("nodes: {names:?}");

        raw_tasks.into_iter().enumerate().for_each(|(i, (name, inputs, mut task))| {

            let mut bytes = [0u8; MAX_TASK_INPUTS];
            
            inputs.iter().enumerate().for_each(|(j, input)| {
                bytes[j] = names.iter().position(|name| *name == *input).expect("Unable to find node associated with input {input}") as u8;
            });

            task.status = TaskStatus::Standby;
            task.link(bytes);
            tm_host.init_node(task);

        });


        let mut ctr = 0;

        while ctr < 10 {

            println!("========= Host Spin");

            let host_to_client = match tm_host.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);

            println!("========== Client Spin");

            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => println!("publish buffer {publish_buffer:?}"),
                None => {},
            };

            ctr += 1;
        }

        assert_eq!(tm_host.n_nodes, tm.n_nodes, "number of nodes did not syncronize");
        for i in 0..tm_host.n_nodes {
            assert_eq!(tm_host.nodes[i].data, tm.nodes[i].data, "TaskConfigs did not syncronize");
        }

        ctr = 0;

        while ctr < tm_host.n_nodes {

            println!("========= Host Panic");

            let host_to_client = match tm_host.panic() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);

            println!("========== Client Spin");

            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => println!("publish buffer {publish_buffer:?}"),
                None => {},
            };

            ctr += 1;
        }

        assert_eq!(0, tm.n_nodes, "number of nodes did not syncronize after panic");

        // for i in 0..tm_host.n_nodes {
        //     tm_host.nodes[i] =
        // }

        ctr = 0;

        while ctr < 10 {

            println!("========= Host Spin");

            let host_to_client = match tm_host.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Host silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            tm.collect(&host_to_client);

            println!("========== Client Spin");

            let client_to_host = match tm.spin() {
                Some(buffer) => buffer,
                None => {
                    println!("Client silent");
                    [0u8; RID_PACKET_SIZE]
                },
            };

            match tm_host.collect(&client_to_host) {
                Some(publish_buffer) => println!("publish buffer {publish_buffer:?}"),
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