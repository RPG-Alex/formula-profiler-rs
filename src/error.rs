use std::io;

/// Project-wide result type.
pub type Result<T> = std::result::Result<T, FormulaProfilerError>;

/// Errors produced by `formula-profiler-rs`.
#[derive(Debug, thiserror::Error)]
pub enum FormulaProfilerError {
    /// Missing arguments
    #[error("arguments must be provided: element name: e.g. 'F' or 'all' for all elements")]
    MissingArguments,
    /// The requested element symbol is not a valid chemical element symbol.
    #[error(
        "invalid element symbol `{symbol}`. Expected a valid chemical element symbol, such as \
         `F`, `Cl`, `Br`, or `I`, or use `all` to profile every observed element"
    )]
    InvalidElementSymbol { symbol: String },

    /// A required summary metric was not found in a generated report table.
    #[error("missing required summary metric `{metric}` in tables/summary.csv")]
    MissingSummaryMetric { metric: &'static str },

    /// A required summary metric could not be parsed.
    #[error("failed to parse summary metric `{metric}` with value `{value}`")]
    InvalidSummaryMetric { metric: &'static str, value: String },

    /// Dataset loading failed.
    #[error("failed to load dataset")]
    DatasetLoad {
        #[source]
        source: Box<dyn std::error::Error>,
    },

    /// CSV reading or writing failed.
    #[error(transparent)]
    Csv(#[from] csv::Error),

    /// Filesystem I/O failed.
    #[error(transparent)]
    Io(#[from] io::Error),

    /// Figure generation failed.
    #[error("failed to render figure: {message}")]
    FigureGeneration { message: String },
}
