use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    error::Result,
    population::{
        PopulationMap, increment_pipe_population, increment_population, summarize_population_map,
        write_population_map_csv,
    },
    records::MoleculeRecord,
    reports::ReportPaths,
    visuals::{write_atom_count_distribution_figure, write_standard_population_figures},
};

#[derive(Debug, Default)]
pub struct ElementProfilerState {
    pub total_records: usize,
    pub records_with_formula: usize,
    pub records_with_target_element: usize,
    pub target_atom_count_distribution: BTreeMap<usize, usize>,
    pub population_maps: BTreeMap<String, PopulationMap>,
}

impl ElementProfilerState {
    pub fn observe(&mut self, record: &MoleculeRecord, target_element: &str) {
        self.total_records += 1;
        self.records_with_formula += 1;

        let target_atom_count = record.atom_count(target_element);
        *self.target_atom_count_distribution.entry(target_atom_count).or_default() += 1;

        let contains_target_element = target_atom_count > 0;
        if contains_target_element {
            self.records_with_target_element += 1;
        }

        for (metadata_group, value) in &record.metadata {
            let counts = self.population_maps.entry(metadata_group.clone()).or_default();
            if value.contains('|') {
                increment_pipe_population(counts, value, contains_target_element);
            } else {
                increment_population(counts, value, contains_target_element);
            }
        }
    }

    pub fn write_reports(&self, target_element: &str, reports: &ReportPaths) -> Result<()> {
        write_summary_csv(
            reports,
            self.total_records,
            self.records_with_formula,
            self.records_with_target_element,
            target_element,
        )?;

        write_atom_count_distribution_csv(
            reports,
            target_element,
            self.records_with_formula,
            &self.target_atom_count_distribution,
        )?;

        write_atom_count_distribution_figure(
            reports,
            target_element,
            self.records_with_formula,
            &self.target_atom_count_distribution,
        )?;

        for (metadata_group, counts) in &self.population_maps {
            let stem = population_stem(metadata_group);
            write_population_outputs(
                reports,
                &stem,
                &format!("{target_element} by {metadata_group}"),
                counts,
                self.total_records,
                self.records_with_target_element,
            )?;
        }

        println!("Total records: {}", self.total_records);
        println!("Records with formula: {}", self.records_with_formula);
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
