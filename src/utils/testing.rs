// Tue Jan 13 2026 - Alex

use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct TestRunner {
    tests: Vec<Test>,
    results: Vec<TestResult>,
    verbose: bool,
}

pub struct Test {
    pub name: String,
    pub category: String,
    pub func: Box<dyn Fn() -> Result<(), String>>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration: Duration,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            results: Vec::new(),
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn add_test<F>(&mut self, name: &str, category: &str, func: F)
    where
        F: Fn() -> Result<(), String> + 'static,
    {
        self.tests.push(Test {
            name: name.to_string(),
            category: category.to_string(),
            func: Box::new(func),
        });
    }

    pub fn run_all(&mut self) -> TestSummary {
        self.results.clear();

        for test in &self.tests {
            let start = Instant::now();
            let result = (test.func)();
            let duration = start.elapsed();

            let (passed, error) = match result {
                Ok(()) => (true, None),
                Err(e) => (false, Some(e)),
            };

            if self.verbose {
                let status = if passed { "PASS" } else { "FAIL" };
                println!("[{}] {} - {:?}", status, test.name, duration);
                if let Some(ref err) = error {
                    println!("  Error: {}", err);
                }
            }

            self.results.push(TestResult {
                name: test.name.clone(),
                category: test.category.clone(),
                passed,
                error,
                duration,
            });
        }

        self.get_summary()
    }

    pub fn run_category(&mut self, category: &str) -> TestSummary {
        self.results.clear();

        for test in &self.tests {
            if test.category != category {
                continue;
            }

            let start = Instant::now();
            let result = (test.func)();
            let duration = start.elapsed();

            let (passed, error) = match result {
                Ok(()) => (true, None),
                Err(e) => (false, Some(e)),
            };

            self.results.push(TestResult {
                name: test.name.clone(),
                category: test.category.clone(),
                passed,
                error,
                duration,
            });
        }

        self.get_summary()
    }

    pub fn get_summary(&self) -> TestSummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        let mut by_category: HashMap<String, (usize, usize)> = HashMap::new();
        for result in &self.results {
            let entry = by_category.entry(result.category.clone()).or_insert((0, 0));
            entry.0 += 1;
            if result.passed {
                entry.1 += 1;
            }
        }

        TestSummary {
            total,
            passed,
            failed,
            duration: total_duration,
            by_category,
            failed_tests: self.results.iter()
                .filter(|r| !r.passed)
                .cloned()
                .collect(),
        }
    }

    pub fn get_results(&self) -> &[TestResult] {
        &self.results
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
    pub by_category: HashMap<String, (usize, usize)>,
    pub failed_tests: Vec<TestResult>,
}

impl TestSummary {
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            1.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }

    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Test Summary ===\n\n");
        report.push_str(&format!("Total:  {}\n", self.total));
        report.push_str(&format!("Passed: {} ({:.1}%)\n", self.passed, self.pass_rate() * 100.0));
        report.push_str(&format!("Failed: {}\n", self.failed));
        report.push_str(&format!("Time:   {:?}\n\n", self.duration));

        if !self.by_category.is_empty() {
            report.push_str("By Category:\n");
            for (category, (total, passed)) in &self.by_category {
                report.push_str(&format!("  {}: {}/{}\n", category, passed, total));
            }
            report.push('\n');
        }

        if !self.failed_tests.is_empty() {
            report.push_str("Failed Tests:\n");
            for test in &self.failed_tests {
                report.push_str(&format!("  - {} [{}]\n", test.name, test.category));
                if let Some(ref err) = test.error {
                    report.push_str(&format!("    Error: {}\n", err));
                }
            }
        }

        report
    }
}

pub fn assert_eq<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), String> {
    if left == right {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} != {:?}", left, right))
    }
}

pub fn assert_ne<T: PartialEq + std::fmt::Debug>(left: T, right: T) -> Result<(), String> {
    if left != right {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} == {:?}", left, right))
    }
}

pub fn assert_true(value: bool) -> Result<(), String> {
    if value {
        Ok(())
    } else {
        Err("Assertion failed: expected true".to_string())
    }
}

pub fn assert_false(value: bool) -> Result<(), String> {
    if !value {
        Ok(())
    } else {
        Err("Assertion failed: expected false".to_string())
    }
}

pub fn assert_some<T>(value: Option<T>) -> Result<T, String> {
    match value {
        Some(v) => Ok(v),
        None => Err("Assertion failed: expected Some, got None".to_string()),
    }
}

pub fn assert_none<T: std::fmt::Debug>(value: Option<T>) -> Result<(), String> {
    match value {
        None => Ok(()),
        Some(v) => Err(format!("Assertion failed: expected None, got Some({:?})", v)),
    }
}

pub fn assert_ok<T, E: std::fmt::Debug>(value: Result<T, E>) -> Result<T, String> {
    match value {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("Assertion failed: expected Ok, got Err({:?})", e)),
    }
}

pub fn assert_err<T: std::fmt::Debug, E>(value: Result<T, E>) -> Result<E, String> {
    match value {
        Err(e) => Ok(e),
        Ok(v) => Err(format!("Assertion failed: expected Err, got Ok({:?})", v)),
    }
}

pub fn assert_contains<T: PartialEq + std::fmt::Debug>(slice: &[T], value: &T) -> Result<(), String> {
    if slice.contains(value) {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} not found in slice", value))
    }
}

pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: T, min: T, max: T) -> Result<(), String> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("Assertion failed: {:?} not in range [{:?}, {:?}]", value, min, max))
    }
}

pub fn create_test_runner() -> TestRunner {
    TestRunner::new()
}
