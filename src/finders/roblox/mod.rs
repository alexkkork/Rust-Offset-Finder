// Tue Jan 13 2026 - Alex

pub mod luau_load;
pub mod new_thread;
pub mod push_instance;
pub mod get_typename;
pub mod identity;
pub mod task_defer;
pub mod task_spawn;
pub mod sctx_resume;
pub mod push_cclosure;
pub mod create_job;
pub mod require_check;
pub mod rbx_crash;
pub mod task_scheduler;

pub use luau_load::LuauLoadFinder;
pub use new_thread::NewThreadFinder;
pub use push_instance::PushInstanceFinder;
pub use get_typename::GetTypenameFinder;
pub use identity::IdentityPropagatorFinder;
pub use task_defer::TaskDeferFinder;
pub use task_spawn::TaskSpawnFinder;
pub use sctx_resume::SctxResumeFinder;
pub use push_cclosure::PushCClosureFinder;
pub use create_job::CreateJobFinder;
pub use require_check::RequireCheckFinder;
pub use rbx_crash::RbxCrashFinder;
pub use task_scheduler::TaskSchedulerFinder;

use crate::memory::{Address, MemoryReader};
use crate::finders::result::FinderResult;
use std::sync::Arc;

pub struct RobloxFinders {
    reader: Arc<dyn MemoryReader>,
}

impl RobloxFinders {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    pub fn find_all(&self, start: Address, end: Address) -> Vec<FinderResult> {
        let mut results = Vec::new();

        if let Some(r) = luau_load::find_luau_load(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = new_thread::find_new_thread(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = push_instance::find_push_instance(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = get_typename::find_get_typename(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = identity::find_identity_propagator(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = task_defer::find_task_defer(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = task_spawn::find_task_spawn(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = sctx_resume::find_sctx_resume(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = push_cclosure::find_push_cclosure(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = create_job::find_create_job(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = require_check::find_require_check(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = rbx_crash::find_rbx_crash(self.reader.clone(), start, end) {
            results.push(r);
        }

        if let Some(r) = task_scheduler::find_task_scheduler(self.reader.clone(), start, end) {
            results.push(r);
        }

        results
    }
}
