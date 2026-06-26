use std::fmt::Debug;

use crate::error::SpectraProfilerError;

pub(crate) fn compact_count(count: usize) -> String {
    match count {
        1_000_000.. => format!("{:.1}M", count as f64 / 1_000_000.0),
        10_000.. => format!("{}k", count / 1_000),
        1_000.. => format!("{:.1}k", count as f64 / 1_000.0),
        _ => count.to_string(),
    }
}

pub(crate) fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64 * 100.0
}

pub(crate) fn figure_error(error: impl Debug) -> SpectraProfilerError {
    SpectraProfilerError::FigureGeneration { message: format!("{error:?}") }
}
