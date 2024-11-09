//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

use alloc::vec;
use alloc::vec::Vec;
pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

/// dead lock stuct
pub struct DeadlockDetect {
    /// available
    pub available: Vec<usize>,
    /// allocation
    pub allocation: Vec<Vec<usize>>,
    /// need
    pub need: Vec<Vec<usize>>,
}
impl Default for DeadlockDetect {
     fn default() -> Self {
        Self::new()
    }
}
impl DeadlockDetect {
    /// new  
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            allocation: vec![Vec::new()],
            need: vec![Vec::new()],
        }
    }
    /// detect
    pub fn detect(&self,tasks:usize,id:usize) -> bool {
        // detect
        let mut work = self.available[id];
        let mut finish = vec![false; tasks];
        for i in 0..tasks {
            finish[i]=self.allocation[i][id]==0;
        }
        loop {
            let mut found=false;
            for i in 0..tasks{
                if !finish[i] && self.need[i][id] <= work {
                    work += self.allocation[i][id];
                    finish[i] = true;
                    found = true;
                }
            }
            if !found{
                break;
            }
        }
        !finish.iter().all(|&x| x)
    }
}