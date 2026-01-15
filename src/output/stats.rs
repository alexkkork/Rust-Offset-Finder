// Tue Jan 13 2026 - Alex

use crate::output::{OffsetOutput, FunctionOffset, StructureOffsets};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct StatisticsCollector {
    start_time: Instant,
    phase_times: HashMap<String, Duration>,
    phase_start: Option<(String, Instant)>,
    counters: HashMap<String, usize>,
    gauges: HashMap<String, f64>,
    histograms: HashMap<String, Vec<f64>>,
    events: Vec<StatEvent>,
    max_events: usize,
}

#[derive(Debug, Clone)]
pub struct StatEvent {
    pub timestamp: Duration,
    pub category: String,
    pub message: String,
    pub value: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CollectedStatistics {
    pub total_duration: Duration,
    pub phase_times: HashMap<String, Duration>,
    pub counters: HashMap<String, usize>,
    pub gauges: HashMap<String, f64>,
    pub histogram_summaries: HashMap<String, HistogramSummary>,
    pub events: Vec<StatEvent>,
}

#[derive(Debug, Clone, Default)]
pub struct HistogramSummary {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub std_dev: f64,
}

impl StatisticsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            phase_times: HashMap::new(),
            phase_start: None,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            events: Vec::new(),
            max_events: 10000,
        }
    }

    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    pub fn start_phase(&mut self, name: &str) {
        if let Some((phase_name, start)) = self.phase_start.take() {
            let duration = start.elapsed();
            *self.phase_times.entry(phase_name).or_insert(Duration::ZERO) += duration;
        }
        self.phase_start = Some((name.to_string(), Instant::now()));
        self.log_event("phase", &format!("Started: {}", name), None);
    }

    pub fn end_phase(&mut self, name: &str) {
        if let Some((phase_name, start)) = self.phase_start.take() {
            let duration = start.elapsed();
            *self.phase_times.entry(phase_name.clone()).or_insert(Duration::ZERO) += duration;
            self.log_event("phase", &format!("Completed: {} ({:.2}ms)", name, duration.as_secs_f64() * 1000.0), None);
        }
    }

    pub fn increment_counter(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn add_to_counter(&mut self, name: &str, value: usize) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn set_counter(&mut self, name: &str, value: usize) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn get_counter(&self, name: &str) -> usize {
        self.counters.get(name).copied().unwrap_or(0)
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn get_gauge(&self, name: &str) -> f64 {
        self.gauges.get(name).copied().unwrap_or(0.0)
    }

    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    pub fn log_event(&mut self, category: &str, message: &str, value: Option<f64>) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }

        self.events.push(StatEvent {
            timestamp: self.start_time.elapsed(),
            category: category.to_string(),
            message: message.to_string(),
            value,
        });
    }

    pub fn record_function_found(&mut self, name: &str, confidence: f64) {
        self.increment_counter("functions_found");
        self.record_histogram("function_confidence", confidence);
        self.log_event("discovery", &format!("Found function: {}", name), Some(confidence));
    }

    pub fn record_structure_found(&mut self, name: &str, size: usize) {
        self.increment_counter("structures_found");
        self.record_histogram("structure_size", size as f64);
        self.log_event("discovery", &format!("Found structure: {} ({} bytes)", name, size), None);
    }

    pub fn record_class_found(&mut self, name: &str) {
        self.increment_counter("classes_found");
        self.log_event("discovery", &format!("Found class: {}", name), None);
    }

    pub fn record_pattern_match(&mut self, pattern_name: &str, address: u64) {
        self.increment_counter("pattern_matches");
        self.log_event("pattern", &format!("Matched: {} @ 0x{:x}", pattern_name, address), None);
    }

    pub fn record_symbol_resolved(&mut self, symbol: &str, address: u64) {
        self.increment_counter("symbols_resolved");
        self.log_event("symbol", &format!("Resolved: {} @ 0x{:x}", symbol, address), None);
    }

    pub fn record_xref_found(&mut self, from: u64, to: u64) {
        self.increment_counter("xrefs_found");
    }

    pub fn record_memory_scanned(&mut self, bytes: usize) {
        self.add_to_counter("memory_scanned", bytes);
    }

    pub fn record_error(&mut self, error: &str) {
        self.increment_counter("errors");
        self.log_event("error", error, None);
    }

    pub fn collect(&self) -> CollectedStatistics {
        let mut histogram_summaries = HashMap::new();

        for (name, values) in &self.histograms {
            histogram_summaries.insert(name.clone(), Self::summarize_histogram(values));
        }

        CollectedStatistics {
            total_duration: self.start_time.elapsed(),
            phase_times: self.phase_times.clone(),
            counters: self.counters.clone(),
            gauges: self.gauges.clone(),
            histogram_summaries,
            events: self.events.clone(),
        }
    }

    fn summarize_histogram(values: &[f64]) -> HistogramSummary {
        if values.is_empty() {
            return HistogramSummary::default();
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = sorted.len();
        let min = sorted[0];
        let max = sorted[count - 1];
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;

        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };

        let p90 = sorted[(count as f64 * 0.90) as usize];
        let p95 = sorted[(count as f64 * 0.95) as usize];
        let p99 = sorted[(count as f64 * 0.99).min((count - 1) as f64) as usize];

        let variance: f64 = sorted.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        HistogramSummary {
            count,
            min,
            max,
            mean,
            median,
            p90,
            p95,
            p99,
            std_dev,
        }
    }

    pub fn from_output(&mut self, output: &OffsetOutput) {
        self.set_counter("total_functions", output.functions.len());
        self.set_counter("total_structures", output.structure_offsets.len());
        self.set_counter("total_classes", output.classes.len());
        self.set_counter("total_properties", output.properties.len());
        self.set_counter("total_methods", output.methods.len());
        self.set_counter("total_constants", output.constants.len());

        for (_, func) in &output.functions {
            self.record_histogram("function_confidence", func.confidence);
        }

        for (_, structure) in &output.structure_offsets {
            self.record_histogram("structure_size", structure.size as f64);
            self.record_histogram("structure_field_count", structure.fields.len() as f64);
        }

        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for (_, func) in &output.functions {
            *category_counts.entry(func.category.clone()).or_insert(0) += 1;
        }
        for (category, count) in category_counts {
            self.set_counter(&format!("category_{}", category), count);
        }
    }

    pub fn format_report(&self) -> String {
        let stats = self.collect();
        let mut report = String::new();

        report.push_str("=== Statistics Report ===\n\n");

        report.push_str(&format!("Total Duration: {:.2}s\n\n", stats.total_duration.as_secs_f64()));

        if !stats.phase_times.is_empty() {
            report.push_str("Phase Times:\n");
            let mut phases: Vec<_> = stats.phase_times.iter().collect();
            phases.sort_by(|a, b| b.1.cmp(a.1));
            for (name, duration) in phases {
                let percent = (duration.as_secs_f64() / stats.total_duration.as_secs_f64()) * 100.0;
                report.push_str(&format!("  {}: {:.2}ms ({:.1}%)\n",
                    name, duration.as_secs_f64() * 1000.0, percent));
            }
            report.push('\n');
        }

        if !stats.counters.is_empty() {
            report.push_str("Counters:\n");
            let mut counters: Vec<_> = stats.counters.iter().collect();
            counters.sort_by(|a, b| a.0.cmp(b.0));
            for (name, value) in counters {
                report.push_str(&format!("  {}: {}\n", name, value));
            }
            report.push('\n');
        }

        if !stats.histogram_summaries.is_empty() {
            report.push_str("Histograms:\n");
            for (name, summary) in &stats.histogram_summaries {
                report.push_str(&format!("  {}:\n", name));
                report.push_str(&format!("    Count: {}\n", summary.count));
                report.push_str(&format!("    Min: {:.2}, Max: {:.2}\n", summary.min, summary.max));
                report.push_str(&format!("    Mean: {:.2}, Median: {:.2}\n", summary.mean, summary.median));
                report.push_str(&format!("    P90: {:.2}, P95: {:.2}, P99: {:.2}\n", summary.p90, summary.p95, summary.p99));
                report.push_str(&format!("    Std Dev: {:.2}\n", summary.std_dev));
            }
            report.push('\n');
        }

        report
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.phase_times.clear();
        self.phase_start = None;
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
        self.events.clear();
    }
}

impl Default for StatisticsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectedStatistics {
    pub fn to_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");

        json.push_str(&format!("  \"total_duration_ms\": {},\n",
            self.total_duration.as_millis()));

        json.push_str("  \"counters\": {\n");
        let counters: Vec<_> = self.counters.iter().collect();
        for (i, (name, value)) in counters.iter().enumerate() {
            let comma = if i < counters.len() - 1 { "," } else { "" };
            json.push_str(&format!("    \"{}\": {}{}\n", name, value, comma));
        }
        json.push_str("  },\n");

        json.push_str("  \"histograms\": {\n");
        let histograms: Vec<_> = self.histogram_summaries.iter().collect();
        for (i, (name, summary)) in histograms.iter().enumerate() {
            let comma = if i < histograms.len() - 1 { "," } else { "" };
            json.push_str(&format!("    \"{}\": {{\n", name));
            json.push_str(&format!("      \"count\": {},\n", summary.count));
            json.push_str(&format!("      \"min\": {:.2},\n", summary.min));
            json.push_str(&format!("      \"max\": {:.2},\n", summary.max));
            json.push_str(&format!("      \"mean\": {:.2},\n", summary.mean));
            json.push_str(&format!("      \"median\": {:.2}\n", summary.median));
            json.push_str(&format!("    }}{}\n", comma));
        }
        json.push_str("  }\n");

        json.push_str("}\n");
        json
    }
}

pub fn create_collector() -> StatisticsCollector {
    StatisticsCollector::new()
}

pub fn collect_from_output(output: &OffsetOutput) -> CollectedStatistics {
    let mut collector = StatisticsCollector::new();
    collector.from_output(output);
    collector.collect()
}
