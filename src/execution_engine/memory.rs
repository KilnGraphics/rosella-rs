use std::collections::VecDeque;
use std::ops::Mul;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult};
use std::thread;
use std::time::Duration;

pub struct AccessGroup {
    semaphore: ash::vk::Semaphore,
    last_access: u64,
}

impl AccessGroup {
    pub fn enqueue_access(&mut self, count: u64) -> AccessInfo {
        let base_access = self.last_access;
        self.last_access += count;

        AccessInfo{ semaphore: self.semaphore, base_access }
    }
}

pub struct AccessInfo {
    pub semaphore: ash::vk::Semaphore,
    pub base_access: u64,
}

type AccessGroupRef = Arc<Mutex<AccessGroup>>;

pub struct AccessGroupSet {
    groups: Vec<AccessGroupRef>,
}

impl AccessGroupSet {
    fn lock_groups(&mut self) -> Result<Vec<MutexGuard<AccessGroup>>, PoisonError<MutexGuard<AccessGroup>>> {
        let mut guards: Vec<MutexGuard<AccessGroup>> = Vec::with_capacity(self.groups.len());
        // Groups **must** be ordered to avoid deadlocking
        for reference in &self.groups {
            match reference.lock() {
                Ok(guard) => guards.push(guard),
                Err(err) => return Err(err),
            }
        }

        Ok(guards)
    }

    pub fn enqueue_access(&mut self) -> Result<Vec<AccessInfo>, &'static str> {
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