use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    error::Result,
    population::{
        PopulationMap, PopulationStats, clean_group_value, split_pipe, summarize_population_map,
        write_population_map_csv,
    },
    records::MoleculeRecord,
    reports::ReportPaths,
    visuals::{write_atom_count_distribution_figure, write_standard_population_figures},
};

#[derive(Debug, Default)]
pub struct GlobalDatasetStats {
    pub total_records: usize,
    pub records_with_formula: usize,
    pub group_value_totals: BTreeMap<String, BTreeMap<String, usize>>,
}

impl GlobalDatasetStats {
    pub fn observe(&mut self, record: &MoleculeRecord) {
        self.total_records += 1;
        self.records_with_formula += 1;

        for (metadata_group, value) in &record.metadata {
            let totals = self.group_value_totals.entry(metadata_group.clone()).or_default();
            if value.contains('|') {
                for part in split_pipe(value) {
                    *totals.entry(clean_group_value(part)).or_default() += 1;
                }
            } else {
                *totals.entry(clean_group_value(value)).or_default() += 1;
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ElementProfilerState {
    pub records_with_target_element: usize,
    pub target_atom_count_distribution: BTreeMap<usize, usize>,
    pub group_value_target_counts: BTreeMap<String, BTreeMap<String, usize>>,
}

impl ElementProfilerState {
    pub fn observe_present(&mut self, record: &MoleculeRecord, target_element: &str) {
        self.records_with_target_element += 1;

        let count = record.atom_count(target_element);
        *self.target_atom_count_distribution.entry(count).or_default() += 1;

        for (metadata_group, value) in &record.metadata {
            let targets = self.group_value_target_counts.entry(metadata_group.clone()).or_default();
            if value.contains('|') {
                for part in split_pipe(value) {
                    *targets.entry(clean_group_value(part)).or_default() += 1;
                }
            } else {
                *targets.entry(clean_group_value(value)).or_default() += 1;
            }
        }
    }

    pub fn write_reports(
        &self,
        target_element: &str,
        global: &GlobalDatasetStats,
        reports: &ReportPaths,
    ) -> Result<()> {
        write_summary_csv(
            reports,
            global.total_records,
            global.records_with_formula,
            self.records_with_target_element,
            target_element,
        )?;

        // Reconstruct the full distribution by dynamically calculating the '0' count
        let mut full_distribution = self.target_atom_count_distribution.clone();
        let zero_count =
            global.records_with_formula.saturating_sub(self.records_with_target_element);
        full_distribution.insert(0, zero_count);

        write_atom_count_distribution_csv(
            reports,
            target_element,
            global.records_with_formula,
            &full_distribution,
        )?;

        write_atom_count_distribution_figure(
            reports,
            target_element,
            global.records_with_formula,
            &full_distribution,
        )?;

        // Combine the global totals and target counts to build the complete
        // PopulationMaps
        for (metadata_group, value_totals) in &global.group_value_totals {
            let stem = population_stem(metadata_group);

            let mut population_map: PopulationMap = BTreeMap::new();
            let target_counts = self.group_value_target_counts.get(metadata_group);

            for (value, &total_count) in value_totals {
                let target_count = target_counts.and_then(|m| m.get(value)).copied().unwrap_or(0);

                population_map.insert(value.clone(), PopulationStats { total_count, target_count });
            }

            write_population_outputs(
                reports,
                &stem,
                &format!("{target_element} by {metadata_group}"),
                &population_map,
                global.total_records,
                self.records_with_target_element,
            )?;
        }

        println!("Total records: {}", global.total_records);
        println!("Records with formula: {}", global.records_with_formula);
        println!("Records with {target_element}: {}", self.records_with_target_element);

        Ok(())
    }
}

fn population_stem(metadata_group: &str) -> String {
    metadata_group.to_ascii_lowercase().replace(' ', "_").replace('/', "_")
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
    records_with_formula: usize,
    records_with_target_element: usize,
    target_element: &str,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("summary.csv"))?;

    writer.write_record(["metric", "value"])?;
    writer.write_record(["target_element".to_string(), target_element.to_string()])?;
    writer.write_record(["total_records".to_string(), total_records.to_string()])?;
    writer.write_record(["records_with_formula".to_string(), records_with_formula.to_string()])?;
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
    percent_of_formula_records: f64,
    contains_target: bool,
}

fn write_atom_count_distribution_csv(
    reports: &ReportPaths,
    target_element: &str,
    records_with_formula: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    let mut writer = csv::Writer::from_path(reports.table("target_atom_count_distribution.csv"))?;

    for (atom_count, record_count) in distribution {
        writer.serialize(AtomCountDistributionRow {
            atom_count: *atom_count,
            record_count: *record_count,
            percent_of_formula_records: percent(*record_count, records_with_formula),
            contains_target: *atom_count > 0,
        })?;
    }

    writer.flush()?;

    println!("Wrote atom-count distribution for {target_element}");

    Ok(())
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64 * 100.0
}
