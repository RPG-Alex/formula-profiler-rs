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
    let unit = record_unit(source);
    let readme_path = dataset_reports_root.as_ref().join("README.md");
    let mut file = File::create(readme_path)?;

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

    writeln!(file)?;
    writeln!(file, "## Dataset facts")?;
    writeln!(file)?;
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|---|---:|")?;
    writeln!(file, "| Profiled {unit} | {} |", profile.record_count())?;
    writeln!(file, "| Observed elements | {} |", profile.observed_element_count())?;

    writeln!(file)?;
    writeln!(file, "## Dataset-level reports")?;
    writeln!(file)?;
    writeln!(
        file,
        "- [Element co-occurrence profile](cooccurrence/README.md): Contains raw and normalized element co-occurrence heatmaps."
    )?;

    writeln!(file)?;
    writeln!(file, "## Observed elements")?;
    writeln!(file)?;
    writeln!(
        file,
        "The following valid chemical elements were observed, ordered by descending frequency."
    )?;
    writeln!(file)?;
    writeln!(file, "`{}`", observed_elements.join("`, `"))?;

    writeln!(file)?;
    writeln!(file, "## Top observed elements")?;
    writeln!(file)?;
    writeln!(file, "| Element | Record count | % of profiled {unit} |")?;
    writeln!(file, "|---|---:|---:|")?;

    for element in observed_elements.iter().take(20) {
        let count = profile.element_count(element);
        let percent_of_records = percent(count, profile.record_count());

        writeln!(file, "| `{element}` | {count} | {percent_of_records:.2}% |")?;
    }

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

    for (element, count) in report_rows {
        let percent_of_records = percent(count, profile.record_count());
        let report_dir = element.to_ascii_lowercase();

        writeln!(
            file,
            "| `{element}` | {count} | {percent_of_records:.2}% | [Open](./{report_dir}/README.md) |"
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
    let unit = record_unit(source);
    let mut file = File::create(reports.readme())?;

    writeln!(file, "# Element co-occurrence profile")?;
    writeln!(file)?;
    writeln!(
        file,
        "This report summarizes which chemical elements appear together in molecular formulas across the dataset."
    )?;
    writeln!(file)?;
    writeln!(file, "## Summary")?;
    writeln!(file)?;
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|---|---:|")?;
    writeln!(file, "| Profiled {unit} | {} |", profile.record_count())?;
    writeln!(file, "| Observed elements | {} |", profile.observed_element_count())?;
    writeln!(file)?;
    writeln!(file, "Heatmap elements shown: `{}`.", heatmap_elements.join("`, `"))?;
    writeln!(file)?;
    writeln!(file, "## Tables")?;
    writeln!(file)?;
    writeln!(file, "- [Element counts](tables/element_counts.csv)")?;
    writeln!(file, "- [Raw co-occurrence counts](tables/element_cooccurrence_counts.csv)")?;
    writeln!(
        file,
        "- [Conditional probabilities](tables/element_cooccurrence_conditional_probability.csv)"
    )?;
    writeln!(file)?;
    writeln!(file)?;
    writeln!(file, "## Heatmaps")?;
    writeln!(file)?;
    writeln!(file, "### Raw co-occurrence counts")?;
    writeln!(file)?;
    writeln!(
        file,
        "<img src=\"figures/element_cooccurrence_raw_counts_heatmap.svg\" alt=\"Raw element co-occurrence heatmap\" />"
    )?;
    writeln!(file)?;
    writeln!(file, "### Conditional probability")?;
    writeln!(file)?;
    writeln!(
        file,
        "<img src=\"figures/element_cooccurrence_conditional_probability_heatmap.svg\" alt=\"Conditional probability element co-occurrence heatmap\" />"
    )?;

    Ok(())
}

fn record_unit(source: &DatasetSource) -> &'static str {
    match source {
        DatasetSource::AnnotatedMs2 | DatasetSource::LocalMgf(_) => "spectra",
        DatasetSource::PubChemSmiles | DatasetSource::Smiles { .. } => "molecules",
    }
}
