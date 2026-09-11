use std::{cmp::Ordering, collections::BTreeMap};

use serde::Deserialize;

use super::sections::POPULATION_TABLES;
use crate::{
    error::{FormulaProfilerError, Result},
    reports::ReportPaths,
};

pub(super) const TOP_ENRICHED_LIMIT: usize = 8;
pub(super) const TOP_ENRICHED_MIN_TOTAL_SUPPORT: usize = 30;

#[derive(Debug)]
pub(super) struct NumericSummary {
    pub(super) total_records: usize,
    pub(super) positive_count: usize,
    pub(super) negative_count: usize,
    pub(super) positive_percentage: f64,
}

#[derive(Debug)]
pub(super) struct EnrichedGroupSummary {
    pub(super) metadata_group: String,
    pub(super) value: String,
    pub(super) total_count: usize,
    pub(super) target_count: usize,
    pub(super) percent_target_within_group: f64,
    pub(super) percent_of_all_target: f64,
}

#[derive(Debug)]
pub(super) struct WarningSummary {
    pub(super) warning: String,
    pub(super) count: usize,
}

#[derive(Debug)]
pub(super) struct MarkdownReportSummary {
    pub(super) numeric: NumericSummary,
    pub(super) top_enriched_groups: Vec<EnrichedGroupSummary>,
    pub(super) warning_summary: Vec<WarningSummary>,
}

#[derive(Debug, Deserialize)]
struct MetricCsvRow {
    metric: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct PopulationCsvRow {
    value: String,
    total_count: usize,
    target_count: usize,
    percent_target_within_group: f64,
    percent_of_all_target: f64,
    support_warning: String,
}

pub(super) fn read_numeric_summary(reports: &ReportPaths) -> Result<NumericSummary> {
    let mut reader = csv::Reader::from_path(reports.table("summary.csv"))?;
    let mut metrics = BTreeMap::new();

    for row in reader.deserialize::<MetricCsvRow>() {
        let row = row?;
        metrics.insert(row.metric, row.value);
    }

    let total_records = read_usize_metric(&metrics, "total_records")?;
    let positive_count = read_usize_metric(&metrics, "records_with_target_element")?;
    let negative_count = total_records.saturating_sub(positive_count);

    let positive_percentage =
        if total_records == 0 { 0.0 } else { positive_count as f64 / total_records as f64 * 100.0 };

    Ok(NumericSummary { total_records, positive_count, negative_count, positive_percentage })
}

pub(super) fn read_markdown_report_summary(reports: &ReportPaths) -> Result<MarkdownReportSummary> {
    Ok(MarkdownReportSummary {
        numeric: read_numeric_summary(reports)?,
        top_enriched_groups: read_top_enriched_groups(reports)?,
        warning_summary: read_warning_summary(reports)?,
    })
}

fn read_usize_metric(metrics: &BTreeMap<String, String>, metric: &'static str) -> Result<usize> {
    let value = metrics.get(metric).ok_or(FormulaProfilerError::MissingSummaryMetric { metric })?;

    value
        .parse()
        .map_err(|_| FormulaProfilerError::InvalidSummaryMetric { metric, value: value.clone() })
}

fn read_top_enriched_groups(reports: &ReportPaths) -> Result<Vec<EnrichedGroupSummary>> {
    let mut groups = Vec::new();

    for table in POPULATION_TABLES {
        for row in read_population_rows(reports, table.filename)? {
            if row.total_count < TOP_ENRICHED_MIN_TOTAL_SUPPORT || row.target_count == 0 {
                continue;
            }

            groups.push(EnrichedGroupSummary {
                metadata_group: table.title.to_string(),
                value: row.value,
                total_count: row.total_count,
                target_count: row.target_count,
                percent_target_within_group: row.percent_target_within_group,
                percent_of_all_target: row.percent_of_all_target,
            });
        }
    }

    groups.sort_by(|left, right| {
        right
            .percent_target_within_group
            .partial_cmp(&left.percent_target_within_group)
            .unwrap_or(Ordering::Equal)
            .then_with(|| right.target_count.cmp(&left.target_count))
            .then_with(|| right.total_count.cmp(&left.total_count))
    });

    groups.truncate(TOP_ENRICHED_LIMIT);

    Ok(groups)
}

fn read_warning_summary(reports: &ReportPaths) -> Result<Vec<WarningSummary>> {
    let mut counts = BTreeMap::<String, usize>::new();

    for table in POPULATION_TABLES {
        for row in read_population_rows(reports, table.filename)? {
            for warning in row
                .support_warning
                .split(['|', ';', ','])
                .map(str::trim)
                .filter(|warning| !warning.is_empty())
                .filter(|warning| *warning != "NONE" && *warning != "OK")
            {
                *counts.entry(warning.to_string()).or_default() += 1;
            }
        }
    }

    Ok(counts.into_iter().map(|(warning, count)| WarningSummary { warning, count }).collect())
}

fn read_population_rows(reports: &ReportPaths, filename: &str) -> Result<Vec<PopulationCsvRow>> {
    let path = reports.table(filename);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut reader = csv::Reader::from_path(path)?;
    let rows =
        reader.deserialize::<PopulationCsvRow>().collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}
