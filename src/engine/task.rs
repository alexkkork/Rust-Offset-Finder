// Tue Jan 13 2026 - Alex

use crate::finders::result::FinderResults;
use crate::memory::MemoryError;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Task {
    id: u64,
    task_type: TaskType,
    priority: TaskPriority,
    timeout: Option<Duration>,
    dependencies: Vec<u64>,
}

impl Task {
    pub fn new(task_type: TaskType) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            task_type,
            priority: TaskPriority::Normal,
            timeout: None,
            dependencies: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_dependency(mut self, dep_id: u64) -> Self {
        self.dependencies.push(dep_id);
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<u64>) -> Self {
        self.dependencies.extend(deps);
        self
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn task_type(&self) -> &TaskType {
        &self.task_type
    }

    pub fn priority(&self) -> TaskPriority {
        self.priority
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    pub fn dependencies(&self) -> &[u64] {
        &self.dependencies
    }

    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }

    pub fn execute(&self) -> Result<FinderResults, TaskError> {
        match self.task_type {
            TaskType::ResolveSymbols => self.execute_resolve_symbols(),
            TaskType::ScanLuaApi => self.execute_scan_lua_api(),
            TaskType::ScanRobloxFunctions => self.execute_scan_roblox_functions(),
            TaskType::ScanBytecode => self.execute_scan_bytecode(),
            TaskType::BuildCallGraph => self.execute_build_call_graph(),
            TaskType::AnalyzeXRefs => self.execute_analyze_xrefs(),
            TaskType::AnalyzeLuaState => self.execute_analyze_lua_state(),
            TaskType::AnalyzeExtraSpace => self.execute_analyze_extraspace(),
            TaskType::AnalyzeClosure => self.execute_analyze_closure(),
            TaskType::AnalyzeProto => self.execute_analyze_proto(),
            TaskType::AnalyzeClasses => self.execute_analyze_classes(),
            TaskType::AnalyzeProperties => self.execute_analyze_properties(),
            TaskType::AnalyzeMethods => self.execute_analyze_methods(),
            TaskType::FindConstants => self.execute_find_constants(),
            TaskType::ValidateResults => self.execute_validate_results(),
            TaskType::Custom(ref name) => self.execute_custom(name),
        }
    }

    fn execute_resolve_symbols(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_scan_lua_api(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_scan_roblox_functions(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_scan_bytecode(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_build_call_graph(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_xrefs(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_lua_state(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_extraspace(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_closure(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_proto(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_classes(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_properties(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_analyze_methods(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_find_constants(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_validate_results(&self) -> Result<FinderResults, TaskError> {
        Ok(FinderResults::new())
    }

    fn execute_custom(&self, name: &str) -> Result<FinderResults, TaskError> {
        Err(TaskError::UnknownTaskType(name.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum TaskType {
    ResolveSymbols,
    ScanLuaApi,
    ScanRobloxFunctions,
    ScanBytecode,
    BuildCallGraph,
    AnalyzeXRefs,
    AnalyzeLuaState,
    AnalyzeExtraSpace,
    AnalyzeClosure,
    AnalyzeProto,
    AnalyzeClasses,
    AnalyzeProperties,
    AnalyzeMethods,
    FindConstants,
    ValidateResults,
    Custom(String),
}

impl TaskType {
    pub fn name(&self) -> &str {
        match self {
            TaskType::ResolveSymbols => "Resolve Symbols",
            TaskType::ScanLuaApi => "Scan Lua API",
            TaskType::ScanRobloxFunctions => "Scan Roblox Functions",
            TaskType::ScanBytecode => "Scan Bytecode",
            TaskType::BuildCallGraph => "Build Call Graph",
            TaskType::AnalyzeXRefs => "Analyze XRefs",
            TaskType::AnalyzeLuaState => "Analyze LuaState",
            TaskType::AnalyzeExtraSpace => "Analyze ExtraSpace",
            TaskType::AnalyzeClosure => "Analyze Closure",
            TaskType::AnalyzeProto => "Analyze Proto",
            TaskType::AnalyzeClasses => "Analyze Classes",
            TaskType::AnalyzeProperties => "Analyze Properties",
            TaskType::AnalyzeMethods => "Analyze Methods",
            TaskType::FindConstants => "Find Constants",
            TaskType::ValidateResults => "Validate Results",
            TaskType::Custom(name) => name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

#[derive(Debug)]
pub enum TaskError {
    MemoryError(MemoryError),
    Timeout,
    Cancelled,
    DependencyFailed(u64),
    UnknownTaskType(String),
    ExecutionError(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::MemoryError(e) => write!(f, "Memory error: {}", e),
            TaskError::Timeout => write!(f, "Task timed out"),
            TaskError::Cancelled => write!(f, "Task was cancelled"),
            TaskError::DependencyFailed(id) => write!(f, "Dependency {} failed", id),
            TaskError::UnknownTaskType(name) => write!(f, "Unknown task type: {}", name),
            TaskError::ExecutionError(e) => write!(f, "Execution error: {}", e),
        }
    }
}

impl std::error::Error for TaskError {}

impl From<MemoryError> for TaskError {
    fn from(e: MemoryError) -> Self {
        TaskError::MemoryError(e)
    }
}
