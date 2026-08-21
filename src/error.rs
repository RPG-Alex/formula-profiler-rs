use std::io;

/// Project-wide result type.
pub type Result<T> = std::result::Result<T, FormulaProfilerError>;

/// Errors produced by `formula-profiler-rs`.
#[derive(Debug, thiserror::Error)]
pub enum FormulaProfilerError {
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

    /// The positional CSV field declaration is missing required fields or
    /// contains duplicate required fields.
    #[error(
        "CSV fields must contain exactly one `id` field and exactly one `smiles` field; found {id_count} `id` fields and {smiles_count} `smiles` fields"
    )]
    InvalidCsvFields { id_count: usize, smiles_count: usize },
}
