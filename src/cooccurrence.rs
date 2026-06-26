use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde::Serialize;

use crate::{
    config::DatasetSource,
    error::Result,
    markdown::cooccurrence::{write_cooccurrence_readme, write_dataset_index_readme},
    records::MoleculeRecord,
    reports::ReportPaths,
    visuals::{write_conditional_probability_heatmap, write_raw_count_heatmap},
};

#[derive(Debug, Default)]
pub struct CooccurrenceProfile {
    pub total_records: usize,
    pub records_with_formula: usize,
    pub element_counts: BTreeMap<String, usize>,
    pub pair_counts: BTreeMap<(String, String), usize>,
}

impl CooccurrenceProfile {
    pub fn observe(&mut self, record: &MoleculeRecord) {
        self.total_records += 1;
        self.records_with_formula += 1;

        let elements: BTreeSet<String> = record.element_counts.keys().cloned().collect();
        self.observe_elements(&elements);
    }

    fn observe_elements(&mut self, elements: &BTreeSet<String>) {
        for element in elements {
            *self.element_counts.entry(element.clone()).or_default() += 1;
        }

        for row_element in elements {
            for column_element in elements {
                *self
                    .pair_counts
                    .entry((row_element.clone(), column_element.clone()))
                    .or_default() += 1;
            }
        }
    }

    pub(crate) fn element_count(&self, element: &str) -> usize {
        self.element_counts.get(element).copied().unwrap_or_default()
    }

    pub(crate) fn pair_count(&self, row_element: &str, column_element: &str) -> usize {
        self.pair_counts
            .get(&(row_element.to_string(), column_element.to_string()))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn conditional_probability(&self, row_element: &str, column_element: &str) -> f64 {
        let row_count = self.element_count(row_element);

        if row_count == 0 {
            return 0.0;
        }

        self.pair_count(row_element, column_element) as f64 / row_count as f64
    }

    fn heatmap_elements(&self) -> Vec<String> {
        let mut elements = self
            .element_counts
            .iter()
            .map(|(element, count)| (element.clone(), *count))
            .collect::<Vec<_>>();

        elements.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        elements.into_iter().map(|(element, _)| element).collect()
    }
}

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

pub fn write_cooccurrence_reports(
    dataset_name: &str,
    profile: &CooccurrenceProfile,
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

fn write_element_counts_csv(profile: &CooccurrenceProfile, reports: &ReportPaths) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("element_counts.csv"))?;

    let mut rows = profile
        .element_counts
        .iter()
        .map(|(element, count)| {
            ElementCountRow {
                element: element.clone(),
                count: *count,
                percent_of_records: percent(*count, profile.records_with_formula),
            }
        })
        .collect::<Vec<_>>();

    rows.sort_by(|left, right| {
        right.count.cmp(&left.count).then_with(|| left.element.cmp(&right.element))
    });

    for row in rows {
        writer.serialize(row)?;
    }

    writer.flush()?;

    Ok(())
}

fn write_cooccurrence_counts_csv(
    profile: &CooccurrenceProfile,
    reports: &ReportPaths,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("element_cooccurrence_counts.csv"))?;
    let elements = profile.element_counts.keys().cloned().collect::<Vec<_>>();

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
    profile: &CooccurrenceProfile,
    reports: &ReportPaths,
) -> Result<()> {
    let mut writer =
        csv::Writer::from_path(reports.table("element_cooccurrence_conditional_probability.csv"))?;
    let elements = profile.element_counts.keys().cloned().collect::<Vec<_>>();

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

pub(crate) fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_probability_handles_zero_denominator() {
        let profile = CooccurrenceProfile::default();

        assert_eq!(profile.conditional_probability("F", "S"), 0.0);
    }
}
