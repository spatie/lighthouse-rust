use std::{io, time::Duration};

/// Failures while configuring, running or reading Lighthouse.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The requested URL is not an absolute HTTP(S) URL.
    #[error("Lighthouse requires an absolute HTTP(S) URL: {0}")]
    InvalidUrl(String),
    /// An unknown category or form factor was supplied.
    #[error("Invalid {kind}: {value}")]
    InvalidEnumValue {
        /// The enum being parsed.
        kind: &'static str,
        /// The rejected value.
        value: String,
    },
    /// A report or configuration has an incompatible shape.
    #[error("Invalid Lighthouse configuration or report: {0}")]
    InvalidData(String),
    /// Node could not start, exited unsuccessfully, or produced invalid output.
    #[error("Could not run Lighthouse: {0}")]
    CouldNotRunLighthouse(String),
    /// Lighthouse completed but could not audit the requested page.
    #[error("Lighthouse reported {code}: {message}")]
    LighthouseReportedError {
        /// Lighthouse's machine-readable runtime error code.
        code: String,
        /// Lighthouse's error message.
        message: String,
    },
    /// The overall process exceeded its configured deadline.
    #[error("Lighthouse timed out after {0:?}")]
    Timeout(Duration),
    /// The requested audit was not present in the report.
    #[error("Audit does not exist: {0}")]
    AuditDoesNotExist(String),
    /// A filesystem or process operation failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON could not be read or written.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
