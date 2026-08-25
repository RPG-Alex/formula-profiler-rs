use std::path::Path;

use serde::Serialize;

use crate::{
    config::DatasetSource,
    error::Result,
    markdown::cooccurrence::{write_cooccurrence_readme, write_dataset_index_readme},
    population::percent,
    profiler::DatasetProfile,
    reports::ReportPaths,
    visuals::{write_conditional_probability_heatmap, write_raw_count_heatmap},
};

#[derive(Debug, Serialize)]
struct ElementCountRow {
    element: String,
    count: usize,
    percent_of_records: f64,
}

#[derive(Debug, Serialize)]
struct CooccurrenceCountRow {
    row_element: String,
    column_element: String,
    cooccurrence_count: usize,
}

#[derive(Debug, Serialize)]
struct ConditionalProbabilityRow {
    row_element: String,
    column_element: String,
    cooccurrence_count: usize,
    row_element_count: usize,
    conditional_probability: f64,
}

pub(crate) fn write_cooccurrence_reports(
    dataset_name: &str,
    profile: &DatasetProfile,
    reports: &ReportPaths,
    dataset_reports_root: impl AsRef<Path>,
    reported_elements: &[String],
    source: &DatasetSource,
) -> Result<()> {
    let heatmap_elements = profile.heatmap_elements();

    write_element_counts_csv(profile, reports)?;
    write_cooccurrence_counts_csv(profile, reports)?;
    write_conditional_probability_csv(profile, reports)?;

    write_raw_count_heatmap(
        reports.figure("element_cooccurrence_raw_counts_heatmap.svg"),
        profile,
        &heatmap_elements,
    )?;

    write_conditional_probability_heatmap(
        reports.figure("element_cooccurrence_conditional_probability_heatmap.svg"),
        profile,
        &heatmap_elements,
    )?;

    write_cooccurrence_readme(reports, profile, &heatmap_elements, source)?;

    write_dataset_index_readme(
        dataset_name,
        dataset_reports_root,
        profile,
        &heatmap_elements,
        reported_elements,
        source,
    )?;

    Ok(())
}

fn write_element_counts_csv(profile: &DatasetProfile, reports: &ReportPaths) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("element_counts.csv"))?;

    for element in profile.heatmap_elements() {
        let count = profile.element_count(&element);

        writer.serialize(ElementCountRow {
            element,
            count,
            percent_of_records: percent(count, profile.record_count()),
        })?;
    }

    writer.flush()?;

    Ok(())
}

fn write_cooccurrence_counts_csv(profile: &DatasetProfile, reports: &ReportPaths) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("element_cooccurrence_counts.csv"))?;
    let elements = profile.observed_elements();

    for row_element in &elements {
        for column_element in &elements {
            writer.serialize(CooccurrenceCountRow {
                row_element: row_element.clone(),
                column_element: column_element.clone(),
                cooccurrence_count: profile.pair_count(row_element, column_element),
            })?;
        }
    }

    writer.flush()?;

    Ok(())
}

fn write_conditional_probability_csv(
    profile: &DatasetProfile,
    reports: &ReportPaths,
) -> Result<()> {
    let mut writer =
        csv::Writer::from_path(reports.table("element_cooccurrence_conditional_probability.csv"))?;
    let elements = profile.observed_elements();

    for row_element in &elements {
        for column_element in &elements {
            writer.serialize(ConditionalProbabilityRow {
                row_element: row_element.clone(),
                column_element: column_element.clone(),
                cooccurrence_count: profile.pair_count(row_element, column_element),
                row_element_count: profile.element_count(row_element),
                conditional_probability: profile
                    .conditional_probability(row_element, column_element),
            })?;
        }
    }

    writer.flush()?;

    Ok(())
}
