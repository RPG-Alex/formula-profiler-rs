use std::{fs::File, io::Write, path::Path};

use crate::{
    config::DatasetSource, error::Result, population::percent, profiler::DatasetProfile,
    reports::ReportPaths,
};

pub(crate) fn write_dataset_index_readme(
    dataset_name: &str,
    dataset_reports_root: impl AsRef<Path>,
    profile: &DatasetProfile,
    observed_elements: &[String],
    reported_elements: &[String],
    source: &DatasetSource,
) -> Result<()> {
    let unit = source.record_unit();
    let readme_path = dataset_reports_root.as_ref().join("README.md");
    let mut file = File::create(readme_path)?;

    write_dataset_intro(&mut file, dataset_name, unit)?;
    write_dataset_facts(&mut file, profile, unit)?;
    write_dataset_report_links(&mut file)?;
    write_observed_elements(&mut file, observed_elements)?;
    write_top_observed_elements(&mut file, profile, observed_elements, unit)?;
    write_generated_element_reports(&mut file, profile, reported_elements, unit)?;

    Ok(())
}

fn write_dataset_intro(file: &mut File, dataset_name: &str, unit: &str) -> Result<()> {
    writeln!(file, "# `{dataset_name}` profiling reports")?;
    writeln!(file)?;
    writeln!(
        file,
        "This directory contains generated exploratory profiling reports for `{dataset_name}`."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "The reports summarize element presence from molecular formulas and should be interpreted as dataset profiling, not direct {unit} evidence."
    )?;
    writeln!(file)?;
    writeln!(
        file,
        "Only records successfully converted into a molecular formula profile are included below; malformed or unparseable inputs are reported during dataset processing."
    )?;

    Ok(())
}

fn write_dataset_facts(file: &mut File, profile: &DatasetProfile, unit: &str) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Dataset facts")?;
    writeln!(file)?;

    write_profile_summary_table(file, profile, unit)
}

fn write_profile_summary_table(
    file: &mut File,
    profile: &DatasetProfile,
    unit: &str,
) -> Result<()> {
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|---|---:|")?;
    writeln!(file, "| Profiled {unit} | {} |", profile.record_count())?;
    writeln!(file, "| Observed elements | {} |", profile.observed_element_count())?;

    Ok(())
}

fn write_dataset_report_links(file: &mut File) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Dataset-level reports")?;
    writeln!(file)?;
    writeln!(
        file,
        "- [Element co-occurrence profile](cooccurrence/README.md): Contains raw and normalized element co-occurrence heatmaps."
    )?;

    Ok(())
}

fn write_observed_elements(file: &mut File, observed_elements: &[String]) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Observed elements")?;
    writeln!(file)?;
    writeln!(
        file,
        "The following valid chemical elements were observed, ordered by descending frequency."
    )?;
    writeln!(file)?;
    writeln!(file, "`{}`", observed_elements.join("`, `"))?;

    Ok(())
}

fn write_top_observed_elements(
    file: &mut File,
    profile: &DatasetProfile,
    observed_elements: &[String],
    unit: &str,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Top observed elements")?;
    writeln!(file)?;
    writeln!(file, "| Element | Record count | % of profiled {unit} |")?;
    writeln!(file, "|---|---:|---:|")?;

    let record_count = profile.record_count();

    for element in observed_elements.iter().take(20) {
        let count = profile.element_count(element);
        let percent_of_records = percent(count, record_count);

        writeln!(file, "| `{element}` | {count} | {percent_of_records:.2}% |")?;
    }

    Ok(())
}

fn write_generated_element_reports(
    file: &mut File,
    profile: &DatasetProfile,
    reported_elements: &[String],
    unit: &str,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Element reports generated in this run")?;
    writeln!(file)?;
    writeln!(
        file,
        "Each element report summarizes metadata groups for profiled {unit} whose formulas contain that element."
    )?;
    writeln!(file)?;
    writeln!(file, "| Element | Record count | % of profiled {unit} | Report |")?;
    writeln!(file, "|---|---:|---:|---|")?;

    let mut report_rows = reported_elements
        .iter()
        .map(|element| (element, profile.element_count(element)))
        .collect::<Vec<_>>();

    report_rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));

    let record_count = profile.record_count();

    for (element, count) in report_rows {
        let percent_of_records = percent(count, record_count);
        let report_dir = element.to_ascii_lowercase();

        writeln!(
            file,
            "| `{element}` | {count} | {percent_of_records:.2}% | \
             [Open](./{report_dir}/README.md) |"
        )?;
    }

    Ok(())
}

pub(crate) fn write_cooccurrence_readme(
    reports: &ReportPaths,
    profile: &DatasetProfile,
    heatmap_elements: &[String],
    source: &DatasetSource,
) -> Result<()> {
    let unit = source.record_unit();
    let mut file = File::create(reports.readme())?;

    write_cooccurrence_intro(&mut file)?;
    write_cooccurrence_summary(&mut file, profile, heatmap_elements, unit)?;
    write_cooccurrence_tables(&mut file)?;
    write_cooccurrence_heatmaps(&mut file)?;

    Ok(())
}

fn write_cooccurrence_intro(file: &mut File) -> Result<()> {
    writeln!(file, "# Element co-occurrence profile")?;
    writeln!(file)?;
    writeln!(
        file,
        "This report summarizes which chemical elements appear together in molecular formulas across the dataset."
    )?;

    Ok(())
}

fn write_cooccurrence_summary(
    file: &mut File,
    profile: &DatasetProfile,
    heatmap_elements: &[String],
    unit: &str,
) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Summary")?;
    writeln!(file)?;

    write_profile_summary_table(file, profile, unit)?;

    writeln!(file)?;
    writeln!(file, "Heatmap elements shown: `{}`.", heatmap_elements.join("`, `"))?;

    Ok(())
}

fn write_cooccurrence_tables(file: &mut File) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Tables")?;
    writeln!(file)?;

    writeln!(file, "- [Element counts](tables/element_counts.csv)")?;

    writeln!(file, "- [Raw co-occurrence counts](tables/element_cooccurrence_counts.csv)")?;

    writeln!(
        file,
        "- [Conditional probabilities](tables/element_cooccurrence_conditional_probability.csv)"
    )?;

    writeln!(file, "- [Normalized PMI](tables/element_cooccurrence_normalized_pmi.csv)")?;

    Ok(())
}

fn write_cooccurrence_heatmaps(file: &mut File) -> Result<()> {
    writeln!(file)?;
    writeln!(file, "## Heatmaps")?;
    writeln!(file)?;

    write_heatmap_section(
        file,
        "Raw co-occurrence counts",
        "figures/element_cooccurrence_raw_counts_heatmap.svg",
        "Raw element co-occurrence heatmap",
        None,
    )?;

    write_heatmap_section(
        file,
        "Conditional probability",
        "figures/element_cooccurrence_conditional_probability_heatmap.svg",
        "Conditional probability element co-occurrence heatmap",
        None,
    )?;

    write_heatmap_section(
        file,
        "Normalized element association (NPMI)",
        "figures/element_cooccurrence_normalized_pmi_heatmap.svg",
        "Normalized PMI element co-occurrence heatmap",
        Some(
            "NPMI measures whether two elements co-occur more or less often than expected from their individual frequencies. Values range from -1 to 1: negative values indicate less co-occurrence than expected, 0 indicates independence, and positive values indicate greater co-occurrence than expected.",
        ),
    )?;

    Ok(())
}

fn write_heatmap_section(
    file: &mut File,
    title: &str,
    image_path: &str,
    alt_text: &str,
    description: Option<&str>,
) -> Result<()> {
    writeln!(file, "### {title}")?;
    writeln!(file)?;

    if let Some(description) = description {
        writeln!(file, "{description}")?;
        writeln!(file)?;
    }

    writeln!(file, "<img src=\"{image_path}\" alt=\"{alt_text}\" />")?;
    writeln!(file)?;

    Ok(())
}
