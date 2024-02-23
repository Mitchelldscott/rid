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

use rid::rtnt::{task_manager::TaskManager};

pub mod rt_task_dead {

    use super::*;

    #[test]
    pub fn rt_task_spawn() {
        let tm = TaskManager::new();
    }

    // #[test]
    // pub fn rt_task_spawn() {
    //     let tm = TaskManager::new();
    // }
}