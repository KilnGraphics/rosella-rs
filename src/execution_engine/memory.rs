use std::collections::VecDeque;
use std::ops::Mul;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult};
use std::thread;
use std::time::Duration;
use crate::rosella::DeviceContext;

use ash::vk;

pub struct AccessGroup {
    device: Arc<DeviceContext>,
    semaphore: vk::Semaphore,
    last_access: Mutex<u64>,
}

impl AccessGroup {
    fn lock_access(&self) -> Result<AccessGuard, &AccessGroup> {
        let guard = self.last_access.lock().map_err(|_| self)?;

        Ok(AccessGuard{ guard, semaphore: self.semaphore })
    }

    pub fn get_counter_value(&self) -> Result<u64, vk::Result> {
        unsafe {
            self.device.get_timeline_semaphore().get_semaphore_counter_value(self.device.vk().handle(), self.semaphore)
        }
    }
}

impl Drop for AccessGroup {
    fn drop(&mut self) {
        unsafe{ self.device.vk().destroy_semaphore(self.semaphore, None) }
    }
}

struct AccessGuard<'a> {
    guard: MutexGuard<'a, u64>,
    semaphore: vk::Semaphore,
}

impl<'a> AccessGuard<'a> {
    pub fn enqueue_access(&mut self, count: u64) -> AccessInfo {
        let base = *self.guard;
        *self.guard += count;

        AccessInfo{ semaphore: self.semaphore, base_access: base }
    }
}

pub struct AccessInfo {
    pub semaphore: ash::vk::Semaphore,
    pub base_access: u64,
}

pub struct AccessGroupSet {
    groups: Vec<Arc<AccessGroup>>,
}

impl AccessGroupSet {
    fn lock_groups(&self) -> Result<Vec<AccessGuard>, &'static str> {
        let mut guards: Vec<AccessGuard> = Vec::with_capacity(self.groups.len());
        // Groups **must** be ordered to avoid deadlocking
        for reference in &self.groups {
            match reference.lock_access() {
                Ok(guard) => guards.push(guard),
                Err(_) => panic!("Poisoned access group"),
            }
        }

        Ok(guards)
    }

    pub fn enqueue_access(&self) -> Result<Vec<AccessInfo>, &'static str> {
        let len = self.groups.len();
        let mut guards = self.lock_groups().ok().ok_or("Poisoned lock in group list")?;
        let mut accesses = Vec::with_capacity(len);

        for guard in &mut guards {
            accesses.push(guard.enqueue_access(1));
        }

        Ok(accesses)
    }

    pub fn enqueue_access_count(&mut self, count: &Vec<u64>) -> Result<Vec<AccessInfo>, &'static str> {
        if count.len() != self.groups.len() {
            panic!("Count vector does not match size of group list");
        }

        let len = self.groups.len();
        let mut guards = self.lock_groups().ok().ok_or("Poisoned lock in group list")?;
        let mut accesses = Vec::with_capacity(len);

        for (i, guard) in guards.iter_mut().enumerate() {
            accesses.push(guard.enqueue_access(*count.get(i).unwrap()));
        }

        Ok(accesses)
    }
}