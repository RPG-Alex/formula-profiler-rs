use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    error::Result,
    population::{
        PopulationMap, PopulationStats, clean_group_value, percent, split_pipe,
        summarize_population_map, write_population_map_csv,
    },
    records::MoleculeRecord,
    reports::ReportPaths,
    visuals::{write_atom_count_distribution_figure, write_standard_population_figures},
};

/// Bin width for Masses
pub(crate) const MASS_BIN_WIDTH_DA: f64 = 25.0;

// should be binning 0-24.999 as bin 0 and so on
fn mass_bin_index(mass_da: f64) -> Option<usize> {
    if !mass_da.is_finite() || mass_da < 0.0 {
        return None;
    }
    Some((mass_da / MASS_BIN_WIDTH_DA).floor() as usize)
}


/// Aggregated statistics collected across all successfully profiled records.
#[derive(Debug, Default)]
pub(crate) struct DatasetProfile {
    /// Number of records successfully added to the profile.
    record_count: usize,
    /// Metadata value counts, key: metadata group, value: counts for each
    /// value.
    group_value_counts: BTreeMap<String, BTreeMap<String, usize>>,
    /// Profiles for observed elements, key: element symbol, value: element
    /// profile.
    elements: BTreeMap<String, ElementProfile>,
    /// Counts for co-occurring elements, key: element-symbol pair, value:
    /// record count.
    pair_counts: BTreeMap<(String, String), usize>,
}

impl DatasetProfile {
    /// Records one accepted molecule once and updates every derived profile
    /// from it.
    pub(crate) fn observe(&mut self, record: &MoleculeRecord) {
        self.record_count += 1;

        let metadata = normalized_metadata(record);
        self.observe_group_values(&metadata);
        self.observe_elements(record, &metadata);
        self.observe_element_pairs(record);
    }

    pub(crate) fn record_count(&self) -> usize {
        self.record_count
    }

    pub(crate) fn observed_elements(&self) -> Vec<String> {
        self.elements.keys().cloned().collect()
    }

    pub(crate) fn observed_element_count(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn element_count(&self, element: &str) -> usize {
        self.elements.get(element).map_or(0, |profile| profile.record_count)
    }

    pub(crate) fn pair_count(&self, left: &str, right: &str) -> usize {
        if left == right {
            return self.element_count(left);
        }

        self.pair_counts.get(&ordered_pair(left, right)).copied().unwrap_or_default()
    }

    pub(crate) fn conditional_probability(&self, row_element: &str, column_element: &str) -> f64 {
        let row_count = self.element_count(row_element);

        if row_count == 0 {
            return 0.0;
        }

        self.pair_count(row_element, column_element) as f64 / row_count as f64
    }

    pub(crate) fn heatmap_elements(&self) -> Vec<String> {
        let mut elements = self
            .elements
            .iter()
            .map(|(element, profile)| (element.clone(), profile.record_count))
            .collect::<Vec<_>>();

        elements.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        elements.into_iter().map(|(element, _)| element).collect()
    }

    pub(crate) fn normalized_pmi(&self, left: &str, right: &str) -> Option<f64> {
        if left == right || self.record_count == 0 {
            return None;
        }

        let left_count = self.element_count(left);
        let right_count = self.element_count(right);

        if left_count == 0 || right_count == 0 {
            return None;
        }
        let pair_count = self.pair_count(left, right);

        if pair_count == 0 {
            return Some(-1.0);
        }
        if pair_count == self.record_count {
            return Some(1.0);
        }
        let total = self.record_count as f64;
        let p_left = left_count as f64 / total;
        let p_right = right_count as f64 / total;
        let p_pair = pair_count as f64 / total;

        let pmi = (p_pair / (p_left * p_right)).ln();
        let npmi = pmi / -p_pair.ln();
        Some(npmi.clamp(-1.0, 1.0))
    }

    pub(crate) fn write_element_reports(
        &self,
        target_element: &str,
        reports: &ReportPaths,
    ) -> Result<()> {
        let empty_profile = ElementProfile::default();
        let element = self.elements.get(target_element).unwrap_or(&empty_profile);

        write_summary_csv(reports, self.record_count, element.record_count, target_element)?;

        let mut full_distribution = element.atom_count_distribution.clone();
        let zero_count = self.record_count.saturating_sub(element.record_count);
        full_distribution.insert(0, zero_count);

        write_atom_count_distribution_csv(
            reports,
            target_element,
            self.record_count,
            &full_distribution,
        )?;

        write_atom_count_distribution_figure(
            reports,
            target_element,
            self.record_count,
            &full_distribution,
        )?;

        for (metadata_group, value_totals) in &self.group_value_counts {
            let stem = population_stem(metadata_group);
            let target_counts = element.group_value_counts.get(metadata_group);
            let mut population_map = PopulationMap::new();

            for (value, &total_count) in value_totals {
                let target_count =
                    target_counts.and_then(|counts| counts.get(value)).copied().unwrap_or(0);

                population_map.insert(value.clone(), PopulationStats { total_count, target_count });
            }

            write_population_outputs(
                reports,
                &stem,
                &format!("{target_element} by {metadata_group}"),
                &population_map,
                self.record_count,
                element.record_count,
            )?;
        }

        println!("Profiled records: {}", self.record_count);
        println!("Records with {target_element}: {}", element.record_count);

        Ok(())
    }

    fn observe_group_values(&mut self, metadata: &NormalizedMetadata<'_>) {
        for (group, values) in metadata {
            let counts = self.group_value_counts.entry((*group).to_string()).or_default();

            for value in values {
                *counts.entry(value.clone()).or_default() += 1;
            }
        }
    }

    fn observe_elements(&mut self, record: &MoleculeRecord, metadata: &NormalizedMetadata<'_>) {
        let mass_bin = mass_bin_index(record.monoisotopic_mass_da);

        for (element, atom_count) in &record.element_counts {
            let profile = self.elements.entry(element.clone()).or_default();
            
            profile.record_count += 1;

            *profile.atom_count_distribution.entry(*atom_count).or_default() += 1;
            
            if let Some(mass_bin) = mass_bin {
                *profile.mass_atom_count_distribution.entry((mass_bin, *atom_count)).or_default() += 1;
            }

            
            for (group, values) in metadata {
                let counts = profile.group_value_counts.entry((*group).to_string()).or_default();

                for value in values {
                    *counts.entry(value.clone()).or_default() += 1;
                }
            }
        }
    }

    fn observe_element_pairs(&mut self, record: &MoleculeRecord) {
        let elements = record.element_counts.keys().collect::<Vec<_>>();

        // Diagonal counts are derived from each element profile, so only distinct pairs
        // are stored.
        for (index, left) in elements.iter().enumerate() {
            for right in elements.iter().skip(index + 1) {
                *self.pair_counts.entry(((*left).clone(), (*right).clone())).or_default() += 1;
            }
        }
    }
}

#[derive(Debug, Default)]
struct ElementProfile {
    record_count: usize,
    atom_count_distribution: BTreeMap<usize, usize>,
    group_value_counts: BTreeMap<String, BTreeMap<String, usize>>,
    // (mass bin, atom count), records
    mass_atom_count_distribution: BTreeMap<(usize, usize), usize>,
}

type NormalizedMetadata<'a> = Vec<(&'a str, Vec<String>)>;

fn normalized_metadata(record: &MoleculeRecord) -> NormalizedMetadata<'_> {
    record
        .metadata
        .iter()
        .map(|(group, value)| (group.as_str(), normalized_group_values(value)))
        .collect()
}

fn normalized_group_values(value: &str) -> Vec<String> {
    if value.contains('|') {
        split_pipe(value).map(clean_group_value).collect()
    } else {
        vec![clean_group_value(value)]
    }
}

fn ordered_pair(left: &str, right: &str) -> (String, String) {
    if left < right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

fn population_stem(metadata_group: &str) -> String {
    metadata_group.to_ascii_lowercase().replace([' ', '/'], "_")
}

fn write_population_outputs(
    reports: &ReportPaths,
    stem: &str,
    title: &str,
    counts: &PopulationMap,
    total_records: usize,
    total_target_records: usize,
) -> Result<()> {
    write_population_map_csv(
        reports.table(&format!("contains_by_{stem}.csv")),
        counts,
        total_records,
        total_target_records,
    )?;

    let summary_rows = summarize_population_map(counts, total_records, total_target_records);
    write_standard_population_figures(reports, stem, title, &summary_rows)?;

    Ok(())
}

fn write_summary_csv(
    reports: &ReportPaths,
    total_records: usize,
    records_with_target_element: usize,
    target_element: &str,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("summary.csv"))?;

    writer.write_record(["metric", "value"])?;
    writer.write_record(["target_element".to_string(), target_element.to_string()])?;
    writer.write_record(["total_records".to_string(), total_records.to_string()])?;
    writer.write_record([
        "records_with_target_element".to_string(),
        records_with_target_element.to_string(),
    ])?;

    writer.flush()?;
    Ok(())
}

#[derive(Debug, Serialize)]
struct AtomCountDistributionRow {
    atom_count: usize,
    record_count: usize,
    percent_of_profiled_records: f64,
    contains_target: bool,
}

fn write_atom_count_distribution_csv(
    reports: &ReportPaths,
    target_element: &str,
    total_records: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("target_atom_count_distribution.csv"))?;

    for (atom_count, record_count) in distribution {
        writer.serialize(AtomCountDistributionRow {
            atom_count: *atom_count,
            record_count: *record_count,
            percent_of_profiled_records: percent(*record_count, total_records),
            contains_target: *atom_count > 0,
        })?;
    }

    writer.flush()?;

    println!("Wrote atom-count distribution for {target_element}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(elements: &[(&str, usize)]) -> MoleculeRecord {
        let element_counts = elements
            .iter()
            .map(|(element, count)| ((*element).to_string(), *count))
            .collect::<BTreeMap<_, _>>();

        MoleculeRecord { element_counts, monoisotopic_mass_da: 0.0,metadata: BTreeMap::new() }
    }

    #[test]
    fn one_observation_updates_record_element_and_pair_counts() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1), ("S", 2)]));
        profile.observe(&record(&[("N", 1)]));

        assert_eq!(profile.record_count(), 2);
        assert_eq!(profile.element_count("N"), 2);
        assert_eq!(profile.element_count("S"), 1);
        assert_eq!(profile.pair_count("N", "S"), 1);
        assert_eq!(profile.pair_count("S", "N"), 1);
        assert_eq!(profile.pair_count("N", "N"), profile.element_count("N"));
        assert_eq!(profile.pair_counts.len(), 1);
    }

    #[test]
    fn conditional_probability_is_column_given_row() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1), ("S", 1)]));
        profile.observe(&record(&[("N", 1)]));

        assert_eq!(profile.conditional_probability("N", "S"), 0.5);
        assert_eq!(profile.conditional_probability("S", "N"), 1.0);
        assert_eq!(profile.conditional_probability("F", "S"), 0.0);
    }

    #[test]
    fn metadata_is_normalized_once_for_global_and_element_counts() {
        let mut molecule = record(&[("N", 1)]);
        molecule.metadata.insert("Class".to_string(), "A | B".to_string());

        let mut profile = DatasetProfile::default();
        profile.observe(&molecule);

        assert_eq!(profile.group_value_counts["Class"]["A"], 1);
        assert_eq!(profile.group_value_counts["Class"]["B"], 1);
        assert_eq!(profile.elements["N"].group_value_counts["Class"]["A"], 1);
        assert_eq!(profile.elements["N"].group_value_counts["Class"]["B"], 1);
    }

    #[test]
    fn normalized_pmi_correct_value() {
        let mut profile = DatasetProfile::default();
        profile.observe(&record(&[("N", 1), ("S", 2)]));
        profile.observe(&record(&[("N", 1)]));

        let npmi = profile.normalized_pmi("N", "S").unwrap_or_else(|| panic!("Value not found!"));

        assert_eq!(npmi, 0.0)
    }

    #[test]
    fn normalized_pmi_is_zero_for_independent_elements() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1), ("S", 1)]));
        profile.observe(&record(&[("N", 1)]));
        profile.observe(&record(&[("S", 1)]));
        profile.observe(&record(&[]));

        let npmi = profile.normalized_pmi("N", "S").unwrap();

        assert!(npmi.abs() < 1e-12);
    }

    #[test]
    fn normalized_pmi_is_one_for_elements_that_always_cooccur() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1), ("S", 1)]));
        profile.observe(&record(&[("N", 1), ("S", 1)]));
        profile.observe(&record(&[]));
        profile.observe(&record(&[]));

        let npmi = profile.normalized_pmi("N", "S").unwrap();

        assert!((npmi - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalized_pmi_is_negative_one_for_elements_that_never_cooccur() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1)]));
        profile.observe(&record(&[("S", 1)]));

        assert_eq!(profile.normalized_pmi("N", "S"), Some(-1.0));
    }

    #[test]
    fn normalized_pmi_is_none_for_same_element() {
        let mut profile = DatasetProfile::default();

        profile.observe(&record(&[("N", 1)]));

        assert_eq!(profile.normalized_pmi("N", "N"), None);
    }

    #[test]
fn molecular_mass_is_assigned_to_expected_bin() {
    assert_eq!(mass_bin_index(0.0), Some(0));
    assert_eq!(mass_bin_index(24.999), Some(0));
    assert_eq!(mass_bin_index(25.0), Some(1));

    assert_eq!(mass_bin_index(124.999), Some(4));
    assert_eq!(mass_bin_index(125.0), Some(5));

    assert_eq!(mass_bin_index(-1.0), None);
    assert_eq!(mass_bin_index(f64::NAN), None);
}
}
