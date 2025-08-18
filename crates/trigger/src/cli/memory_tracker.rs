use std::sync::{Arc, Mutex, OnceLock};
use spin_core::async_trait;
use spin_factors::RuntimeFactors;
use spin_factors_executor::{ExecutorHooks, FactorsInstanceBuilder};

static GLOBAL_MEMORY_TRACKER: OnceLock<MemoryTracker> = OnceLock::new();

/// A global memory tracker that tracks peak memory usage across all instances
#[derive(Clone)]
pub struct MemoryTracker {
    peak_memory: Arc<Mutex<u64>>,
    current_memory: Arc<Mutex<u64>>,
    instance_count: Arc<Mutex<u64>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            peak_memory: Arc::new(Mutex::new(0)),
            current_memory: Arc::new(Mutex::new(0)),
            instance_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get or initialize the global memory tracker
    pub fn global() -> &'static MemoryTracker {
        GLOBAL_MEMORY_TRACKER.get_or_init(|| MemoryTracker::new())
    }

    pub fn update_memory(&self, memory_usage: u64) {
        {
            let mut current = self.current_memory.lock().unwrap();
            *current = memory_usage;
        }
        {
            let mut peak = self.peak_memory.lock().unwrap();
            if memory_usage > *peak {
                *peak = memory_usage;
            }
        }
    }

    pub fn get_peak_memory(&self) -> u64 {
        *self.peak_memory.lock().unwrap()
    }

    pub fn get_current_memory(&self) -> u64 {
        *self.current_memory.lock().unwrap()
    }

    pub fn increment_instance_count(&self) {
        *self.instance_count.lock().unwrap() += 1;
    }

    pub fn get_instance_count(&self) -> u64 {
        *self.instance_count.lock().unwrap()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// An [`ExecutorHooks`] that tracks peak memory usage across all instances
pub struct MemoryTrackerHook;

impl MemoryTrackerHook {
    pub fn new() -> Self {
        Self
    }

    pub fn tracker() -> &'static MemoryTracker {
        MemoryTracker::global()
    }
}

#[async_trait]
impl<F: RuntimeFactors, U> ExecutorHooks<F, U> for MemoryTrackerHook {
    fn prepare_instance(&self, _builder: &mut FactorsInstanceBuilder<F, U>) -> anyhow::Result<()> {
        // Track each instance creation
        MemoryTracker::global().increment_instance_count();
        Ok(())
    }
}
