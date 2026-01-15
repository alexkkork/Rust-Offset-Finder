// Tue Jan 15 2026 - Alex

pub mod binary;
pub mod offset;
pub mod report;
pub mod version;
pub mod analyzer;

pub use binary::{BinaryDiff, BinaryChange, ChangeKind, DiffRegion};
pub use offset::{OffsetDiff, OffsetChange, OffsetMigration, MigrationStrategy};
pub use report::{DiffReport, DiffReportBuilder, ReportFormat};
pub use version::{Version, VersionInfo, VersionComparison};
pub use analyzer::{DiffAnalyzer, DiffResult, DiffSummary};
