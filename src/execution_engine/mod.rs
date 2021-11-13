use std::sync::{Arc, LockResult, Mutex};
use crate::rosella::DeviceContext;

use ash::vk;
use crate::execution_engine::executable::ExecutableInternal;
use crate::init::device::VulkanQueue;

pub mod commands;
pub mod ops;
pub mod ops_compile;
pub mod placeholder_objects;
pub mod memory;
pub mod executable;

mod object_manager;
mod resource_state;
mod static_resource_state;

mod keep_alive {
    use std::sync::{Arc, Mutex};
    use std::sync::atomic::AtomicBool;
    use std::thread::{JoinHandle, Thread};
    use crate::execution_engine::executable::ExecutableInternal;
    use crate::execution_engine::memory::AccessGroup;

    use ash::vk;

    pub struct WaitTask {
        pub access_group: Arc<AccessGroup>,
        pub wait_value: u64,
    }

    pub type WaitSet = Box<[WaitTask]>;

    struct Entry {
        wait_set: WaitSet,
        payload: Arc<Mutex<ExecutableInternal>>,
    }

    impl Entry {
        fn is_entry_done(&self) -> Result<bool, vk::Result> {
            for wait in self.wait_set.iter() {
                if wait.access_group.get_counter_value()? < wait.wait_value {
                    return Ok(false);
                }
            }

            Ok(true)
        }
    }

    struct KeepAliveServiceInternal {
        tasks: Mutex<Vec<Entry>>,
        kill: AtomicBool,
    }

    impl KeepAliveServiceInternal {
        fn run_validate(&mut self) {

        }

        fn is_empty(&self) -> bool {
            self.tasks.lock()
        }
    }

    pub struct KeepAliveService {
        internal: Arc<KeepAliveServiceInternal>,
        worker: JoinHandle<()>,
    }

    impl KeepAliveService {
        fn run(service: Arc<KeepAliveServiceInternal>) {
            loop {



            }
        }

        pub fn start() -> Self {
            let internal = Arc::new(KeepAliveServiceInternal{ tasks: Mutex::new(Vec::with_capacity(8)), kill: AtomicBool::new(false)});

            let worker_internal = internal.clone();
            let worker = std::thread::spawn(|| Self::run(worker_internal));

            Self{ internal, worker }
        }

        pub fn add_task(&mut self, payload: Arc<Mutex<ExecutableInternal>>, wait_set: WaitSet) {
            let mut tasks = self.internal.tasks.lock().unwrap();
            tasks.push(Entry{ payload, wait_set });
        }
    }
}

pub use keep_alive::{WaitTask, WaitSet};

pub struct ExecutionEngine {
    device: Arc<DeviceContext>,
    command_pools: Box<[Mutex<vk::CommandPool>]>,
    queues: Box<[Arc<VulkanQueue>]>,
}

impl ExecutionEngine {
    pub fn new(device: Arc<DeviceContext>, queues: Box<[Arc<VulkanQueue>]>) -> Result<Self, vk::Result> {
        let mut command_pools = Vec::new();
        command_pools.resize_with(queues.len(), || Mutex::new(vk::CommandPool::null()));

        for (i, queue) in queues.iter().enumerate() {
            if i != queue.get_queue_family_index() as usize {
                panic!("Yes this is not very good TODO fix this") // TODO fix this
            }

            let create_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(i as u32);

            let pool = unsafe{ device.vk().create_command_pool(&create_info.build(), None) }?;
            *command_pools.get_mut(i).unwrap().get_mut().unwrap() = pool;
        }

        Ok(Self{ device, queues, command_pools: command_pools.into_boxed_slice() })
    }

    fn get_device(&self) -> &DeviceContext {
        self.device.as_ref()
    }

    fn get_queues(&self) -> &[Arc<VulkanQueue>] {
        self.queues.as_ref()
    }

    fn get_command_pools(&self) -> &[Mutex<vk::CommandPool>] {
        self.command_pools.as_ref()
    }

    fn add_keep_alive(&self, payload: Arc<Mutex<ExecutableInternal>>, wait_set: WaitSet) {

    }
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        for pool in self.command_pools.iter_mut() {
            let pool = match pool.get_mut() {
                Ok(p) => p,
                Err(err) => err.into_inner()
            };

            unsafe{ self.device.vk().destroy_command_pool(*pool, None) };
        }
    }
}