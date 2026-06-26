pub mod cooccurrence;
mod data;
mod render;
mod sections;

use std::{fs::File, io::Write};

use self::{
    data::read_markdown_report_summary,
    render::{
        write_atom_count_distribution_section, write_glossary_and_references,
        write_interpretation_guide, write_numeric_summary, write_report_links,
        write_top_enriched_groups, write_warning_summary,
    },
    sections::{REPORT_SECTIONS, write_population_section},
};
use crate::{config::DatasetSource, error::Result, reports::ReportPaths};

pub fn write_markdown_report(
    dataset_name: &str,
    target_element: &str,
    reports: &ReportPaths,
    source: &DatasetSource,
) -> Result<()> {
    let mut file = File::create(reports.readme())?;
    let summary = read_markdown_report_summary(reports)?;

    writeln!(file, "# `{target_element}` profile for `{dataset_name}`")?;
    writeln!(file)?;
    writeln!(
        file,
        "This report summarizes how often the target element `{target_element}` appears across metadata groups in `{dataset_name}`.",
    )?;

    write_interpretation_guide(&mut file, source, target_element)?;
    write_glossary_and_references(&mut file, source)?;
    write_numeric_summary(&mut file, &summary.numeric, source)?;
    write_atom_count_distribution_section(&mut file, source, target_element)?;
    write_top_enriched_groups(&mut file, &summary.top_enriched_groups, source)?;
    write_warning_summary(&mut file, source, &summary.warning_summary)?;
    write_report_links(&mut file, source)?;

    for section in REPORT_SECTIONS {
        write_population_section(&mut file, reports, section)?;
    }

    Ok(())
}
