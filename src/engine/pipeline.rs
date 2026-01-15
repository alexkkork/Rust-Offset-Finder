// Tue Jan 13 2026 - Alex

use crate::engine::task::Task;
use crate::engine::stage::Stage;

pub struct Pipeline {
    stages: Vec<Stage>,
    current_stage: usize,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            current_stage: 0,
        }
    }

    pub fn add_stage(&mut self, stage: Stage) {
        self.stages.push(stage);
    }

    pub fn add_stage_with_tasks(&mut self, name: &str, tasks: Vec<Task>) {
        let mut stage = Stage::new(name.to_string());
        for task in tasks {
            stage.add_task(task);
        }
        self.stages.push(stage);
    }

    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    pub fn stages_mut(&mut self) -> &mut [Stage] {
        &mut self.stages
    }

    pub fn current_stage(&self) -> Option<&Stage> {
        self.stages.get(self.current_stage)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_stage < self.stages.len() - 1 {
            self.current_stage += 1;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.current_stage = 0;
        for stage in &mut self.stages {
            stage.reset();
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_stage >= self.stages.len()
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    pub fn current_stage_index(&self) -> usize {
        self.current_stage
    }

    pub fn progress(&self) -> f64 {
        if self.stages.is_empty() {
            1.0
        } else {
            self.current_stage as f64 / self.stages.len() as f64
        }
    }

    pub fn total_tasks(&self) -> usize {
        self.stages.iter().map(|s| s.task_count()).sum()
    }

    pub fn insert_stage(&mut self, index: usize, stage: Stage) {
        if index <= self.stages.len() {
            self.stages.insert(index, stage);

            if index <= self.current_stage {
                self.current_stage += 1;
            }
        }
    }

    pub fn remove_stage(&mut self, index: usize) -> Option<Stage> {
        if index < self.stages.len() {
            let stage = self.stages.remove(index);

            if index < self.current_stage {
                self.current_stage = self.current_stage.saturating_sub(1);
            }

            Some(stage)
        } else {
            None
        }
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PipelineBuilder {
    pipeline: Pipeline,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            pipeline: Pipeline::new(),
        }
    }

    pub fn stage(mut self, name: &str) -> StageBuilder {
        StageBuilder {
            pipeline_builder: self,
            stage: Stage::new(name.to_string()),
        }
    }

    pub fn build(self) -> Pipeline {
        self.pipeline
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StageBuilder {
    pipeline_builder: PipelineBuilder,
    stage: Stage,
}

impl StageBuilder {
    pub fn task(mut self, task: Task) -> Self {
        self.stage.add_task(task);
        self
    }

    pub fn tasks(mut self, tasks: Vec<Task>) -> Self {
        for task in tasks {
            self.stage.add_task(task);
        }
        self
    }

    pub fn done(mut self) -> PipelineBuilder {
        self.pipeline_builder.pipeline.add_stage(self.stage);
        self.pipeline_builder
    }
}

pub fn create_default_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .stage("Symbol Resolution")
            .task(Task::new(crate::engine::task::TaskType::ResolveSymbols))
            .done()
        .stage("Pattern Scanning")
            .task(Task::new(crate::engine::task::TaskType::ScanLuaApi))
            .task(Task::new(crate::engine::task::TaskType::ScanRobloxFunctions))
            .task(Task::new(crate::engine::task::TaskType::ScanBytecode))
            .done()
        .stage("XRef Analysis")
            .task(Task::new(crate::engine::task::TaskType::BuildCallGraph))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeXRefs))
            .done()
        .stage("Structure Analysis")
            .task(Task::new(crate::engine::task::TaskType::AnalyzeLuaState))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeExtraSpace))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeClosure))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeProto))
            .done()
        .stage("Class Analysis")
            .task(Task::new(crate::engine::task::TaskType::AnalyzeClasses))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeProperties))
            .task(Task::new(crate::engine::task::TaskType::AnalyzeMethods))
            .done()
        .stage("Constant Analysis")
            .task(Task::new(crate::engine::task::TaskType::FindConstants))
            .done()
        .stage("Validation")
            .task(Task::new(crate::engine::task::TaskType::ValidateResults))
            .done()
        .build()
}
