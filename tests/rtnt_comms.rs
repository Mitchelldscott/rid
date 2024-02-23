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

use rid::{MAX_TASK_DATA_BYTES, MAX_TASK_CONFIG_CHUNKS, rtnt::{task_generator::TaskDriver, task_manager::{TaskNode, TaskConfig, TaskManager}}};

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
        pub inputs: u32,
        pub driver: u8,
        pub data: Vec<T>
    }

    impl<T: serde::de::DeserializeOwned + ToBeBytes> SerializableNode<T> {

        pub fn load(data: &str) -> TaskNode {

            let node: SerializableNode<T> = toml::from_str(data).unwrap();
            TaskNode::new(node.stream, node.rate, node.inputs, TaskDriver::new(node.driver), node.data_as_config())

        }

        pub fn data_as_config(&self)-> TaskConfig {

            let mut buffer = [[0u8; MAX_TASK_DATA_BYTES]; MAX_TASK_CONFIG_CHUNKS];
            let mut element = 0;
            let mut chunk = 0;

            self.data.iter().for_each(|value| {
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

            TaskConfig::new(chunk, buffer)
            
        }
    }

    #[test]
    pub fn rtnt_node_dump() {
        let tm = TaskManager::default();
        // let toml = toml::to_string(&tm.nodes[0]).unwrap();
        // println!("{toml}");
        // let tm_host = TaskManager::new(toml::from_file())
        let toml_data = r#"
            [switch]
            stream = false
            rate = 100
            inputs = 0
            driver = 1
            data = [1.0, 2.0, 3.0]
        "#;

        let data: toml::Value = toml::from_str(toml_data).unwrap();
        // let node = SerializableNode::<f32>::load(toml_data);

        println!("{data:?}")
    }

    // #[test]
    // pub fn rt_task_spawn() {
    //     let tm = TaskManager::new();
    // }
}