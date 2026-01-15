// Tue Jan 15 2026 - Alex

pub mod validator;
pub mod rules;
pub mod checker;
pub mod report;
pub mod confidence;
pub mod pointer_validation;
pub mod cross_validation;
pub mod size_validation;

pub use validator::OffsetValidator;
pub use rules::ValidationRule;
pub use checker::ValidationChecker;
pub use report::{ValidationReport, ValidationIssue, IssueSeverity};
pub use confidence::ConfidenceScorer;
pub use pointer_validation::{PointerValidator, PointerValidationConfig, PointerValidationResult, PointerIssue, PointerExpectation};
pub use cross_validation::{CrossValidator, CrossValidationCheck, CrossValidationReport, CheckResult, ResultAggregator, AggregatedResult};
pub use size_validation::{SizeValidator, ExpectedSize, SizeValidationResult, InferredSize, AlignmentValidation};
