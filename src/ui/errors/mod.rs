// Wed Jan 15 2026 - Alex

pub mod display;
pub mod handler;
pub mod reporter;

pub use display::ErrorDisplay;
pub use handler::ErrorHandler;
pub use reporter::ErrorReporter;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Display error: {0}")]
    Display(String),

    #[error("Progress error: {0}")]
    Progress(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Theme error: {0}")]
    Theme(String),
}

pub type UiResult<T> = Result<T, UiError>;
