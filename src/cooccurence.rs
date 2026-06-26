use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Write,
    path::Path,
};

use plotters::prelude::*;
use serde::Serialize;

use crate::{
    error::{Result, SpectraProfilerError},
    records::MoleculeRecord,
    reports::ReportPaths,
};

// Make this struct public so main.rs can instantiate it
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

    fn element_count(&self, element: &str) -> usize {
        self.element_counts.get(element).copied().unwrap_or_default()
    }

    fn pair_count(&self, row_element: &str, column_element: &str) -> usize {
        self.pair_counts
            .get(&(row_element.to_string(), column_element.to_string()))
            .copied()
            .unwrap_or_default()
    }

    fn conditional_probability(&self, row_element: &str, column_element: &str) -> f64 {
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
    profile: &CooccurrenceProfile, // CHANGED from dataset: &LoadedDataset
    reports: &ReportPaths,
    dataset_reports_root: impl AsRef<Path>,
    reported_elements: &[String],
) -> Result<()> {
    let heatmap_elements = profile.heatmap_elements();

    write_element_counts_csv(profile, reports)?;
    write_cooccurrence_counts_csv(profile, reports)?;
    write_conditional_probability_csv(profile, reports)?;

    render_raw_count_heatmap(
        reports.figure("element_cooccurrence_raw_counts_heatmap.svg"),
        profile,
        &heatmap_elements,
    )?;

    render_conditional_probability_heatmap(
        reports.figure("element_cooccurrence_conditional_probability_heatmap.svg"),
        profile,
        &heatmap_elements,
    )?;

    write_cooccurrence_readme(reports, profile, &heatmap_elements)?;

    write_dataset_index_readme(
        dataset_name,
        dataset_reports_root,
        profile,
        &heatmap_elements,
        reported_elements,
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

fn render_raw_count_heatmap(
    path: impl AsRef<Path>,
    profile: &CooccurrenceProfile,
    elements: &[String],
) -> Result<()> {
    let values = elements
        .iter()
        .flat_map(|row| elements.iter().map(move |column| profile.pair_count(row, column) as f64))
        .collect::<Vec<_>>();

    let max_log_value =
        values.iter().map(|value| (value + 1.0).log10()).fold(0.0_f64, f64::max).max(1.0);

    render_heatmap(path, "Element co-occurrence counts", elements, |row, column| {
        let count = profile.pair_count(row, column);
        let scaled = ((count as f64 + 1.0).log10() / max_log_value).clamp(0.0, 1.0);

        (scaled, compact_count(count))
    })
}

fn render_conditional_probability_heatmap(
    path: impl AsRef<Path>,
    profile: &CooccurrenceProfile,
    elements: &[String],
) -> Result<()> {
    render_heatmap(path, "Element co-occurrence probability", elements, |row, column| {
        let probability = profile.conditional_probability(row, column);

        (probability.clamp(0.0, 1.0), format!("{:.0}%", probability * 100.0))
    })
}

fn render_heatmap<F>(
    path: impl AsRef<Path>,
    title: &str,
    elements: &[String],
    value_for: F,
) -> Result<()>
where
    F: Fn(&str, &str) -> (f64, String),
{
    if elements.is_empty() {
        return Ok(());
    }

    let cell_size = heatmap_cell_size(elements.len());
    let label_font_size = heatmap_font_size(cell_size, 0.35, 8, 22);
    let value_font_size = heatmap_font_size(cell_size, 0.28, 6, 18);

    let left_margin = 120_i32;
    let top_margin = 80_i32 + label_font_size * 3;
    let right_margin = 60_i32;
    let bottom_margin = 50_i32;

    let width = left_margin + right_margin + cell_size * elements.len() as i32;
    let height = top_margin + bottom_margin + cell_size * elements.len() as i32;

    let heatmap_width = cell_size * elements.len() as i32;
    let heatmap_center_x = left_margin + heatmap_width / 2;

    let root = SVGBackend::new(path.as_ref(), (width as u32, height as u32)).into_drawing_area();

    root.fill(&WHITE).map_err(figure_error)?;

    let title_font_size = 34_i32;
    let estimated_title_width = title.chars().count() as i32 * title_font_size / 2;
    let title_x = heatmap_center_x - estimated_title_width / 2;

    root.draw(&Text::new(title, (title_x, 42), ("sans-serif", title_font_size).into_font()))
        .map_err(figure_error)?;

    for (index, element) in elements.iter().enumerate() {
        let index = index as i32;
        let x = left_margin + index * cell_size + cell_size / 2;
        let y = top_margin + index * cell_size + cell_size / 2;

        let column_label_style = ("sans-serif", label_font_size).into_font();

        root.draw(&Text::new(
            element.clone(),
            (x - label_font_size / 4, top_margin - 18),
            column_label_style,
        ))
        .map_err(figure_error)?;

        root.draw(&Text::new(
            element.clone(),
            (left_margin - 28, y + label_font_size / 5),
            ("sans-serif", label_font_size).into_font(),
        ))
        .map_err(figure_error)?;
    }

    for (row_index, row_element) in elements.iter().enumerate() {
        for (column_index, column_element) in elements.iter().enumerate() {
            let row_index = row_index as i32;
            let column_index = column_index as i32;

            let x0 = left_margin + column_index * cell_size;
            let y0 = top_margin + row_index * cell_size;
            let x1 = x0 + cell_size;
            let y1 = y0 + cell_size;

            let (scaled_value, label) = value_for(row_element, column_element);
            let color = heatmap_color(scaled_value);

            root.draw(&Rectangle::new([(x0, y0), (x1, y1)], color.filled()))
                .map_err(figure_error)?;

            root.draw(&Rectangle::new(
                [(x0, y0), (x1, y1)],
                ShapeStyle::from(&WHITE.mix(0.85)).stroke_width(1),
            ))
            .map_err(figure_error)?;

            let text_color = if scaled_value > 0.58 { WHITE } else { BLACK };

            root.draw(&Text::new(
                label,
                (x0 + 6, y0 + cell_size / 2 + value_font_size / 3),
                ("sans-serif", value_font_size).into_font().color(&text_color),
            ))
            .map_err(figure_error)?;
        }
    }

    root.present().map_err(figure_error)?;

    Ok(())
}

fn heatmap_color(value: f64) -> RGBColor {
    let value = value.clamp(0.0, 1.0);

    let red = (255.0 * value) as u8;
    let green = (245.0 * (1.0 - (value * 0.65))) as u8;
    let blue = (255.0 * (1.0 - value)) as u8;

    RGBColor(red, green, blue)
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64 * 100.0
}

fn compact_count(count: usize) -> String {
    match count {
        1_000_000.. => format!("{:.1}M", count as f64 / 1_000_000.0),
        10_000.. => format!("{}k", count / 1_000),
        1_000.. => format!("{:.1}k", count as f64 / 1_000.0),
        _ => count.to_string(),
    }
}

fn figure_error(error: impl std::fmt::Debug) -> SpectraProfilerError {
    SpectraProfilerError::FigureGeneration { message: format!("{error:?}") }
}

fn write_cooccurrence_readme(
    reports: &ReportPaths,
    profile: &CooccurrenceProfile,
    heatmap_elements: &[String],
) -> Result<()> {
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
    writeln!(file, "| Total spectra | {} |", profile.total_records)?;
    writeln!(file, "| Spectra with formula | {} |", profile.records_with_formula)?;
    writeln!(file, "| Observed elements | {} |", profile.element_counts.len())?;
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

fn heatmap_cell_size(element_count: usize) -> i32 {
    match element_count {
        0..=10 => 72,
        11..=16 => 58,
        17..=24 => 46,
        25..=36 => 36,
        37..=50 => 28,
        _ => 22,
    }
}

fn heatmap_font_size(cell_size: i32, scale: f64, min: i32, max: i32) -> i32 {
    ((cell_size as f64 * scale).round() as i32).clamp(min, max)
}

fn write_dataset_index_readme(
    dataset_name: &str,
    dataset_reports_root: impl AsRef<Path>,
    profile: &CooccurrenceProfile,
    observed_elements: &[String],
    reported_elements: &[String],
) -> Result<()> {
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
        "The reports summarize element presence from molecular formula metadata and should be \
         interpreted as dataset profiling, not direct spectral evidence."
    )?;

    writeln!(file)?;
    writeln!(file, "## Dataset facts")?;
    writeln!(file)?;
    writeln!(file, "| Metric | Value |")?;
    writeln!(file, "|---|---:|")?;
    writeln!(file, "| Total spectra | {} |", profile.total_records)?;
    writeln!(file, "| Spectra with formula metadata | {} |", profile.records_with_formula)?;
    writeln!(
        file,
        "| Spectra without formula metadata | {} |",
        profile.total_records.saturating_sub(profile.records_with_formula)
    )?;
    writeln!(file, "| Observed elements | {} |", profile.element_counts.len())?;

    writeln!(file)?;
    writeln!(file, "## Dataset-level reports")?;
    writeln!(file)?;
    writeln!(
        file,
        "- [Element co-occurrence profile](cooccurrence/README.md): Contains raw and normalized atom co-occurrence heatmaps."
    )?;

    writeln!(file)?;
    writeln!(file, "## Observed elements")?;
    writeln!(file)?;
    writeln!(
        file,
        "The following valid chemical elements were observed in molecular formula metadata, \
         ordered by descending frequency."
    )?;
    writeln!(file)?;
    writeln!(file, "`{}`", observed_elements.join("`, `"))?;

    writeln!(file)?;
    writeln!(file, "## Top observed elements")?;
    writeln!(file)?;
    writeln!(file, "| Element | Formula count | % of formula-bearing spectra |")?;
    writeln!(file, "|---|---:|---:|")?;

    for element in observed_elements.iter().take(20) {
        let count = profile.element_count(element);
        let percent_of_formula_records = percent(count, profile.records_with_formula);

        writeln!(file, "| `{element}` | {count} | {percent_of_formula_records:.2}% |")?;
    }

    writeln!(file)?;
    writeln!(file, "## Element reports generated in this run")?;
    writeln!(file)?;
    writeln!(
        file,
        "Each element report summarizes metadata groups for spectra whose formulas contain that element."
    )?;
    writeln!(file)?;
    writeln!(file, "| Element | Formula count | % of formula-bearing spectra | Report |")?;
    writeln!(file, "|---|---:|---:|---|")?;

    let mut report_rows = reported_elements
        .iter()
        .map(|element| {
            let count = profile.element_count(element);
            (element, count)
        })
        .collect::<Vec<_>>();

    report_rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));

    for (element, count) in report_rows {
        let percent_of_formula_records = percent(count, profile.records_with_formula);
        let report_dir = element.to_ascii_lowercase();

        writeln!(
            file,
            "| `{element}` | {count} | {percent_of_formula_records:.2}% | [Open](./{report_dir}/README.md) |"
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_count_formats_large_counts() {
        assert_eq!(compact_count(42), "42");
        assert_eq!(compact_count(1_500), "1.5k");
        assert_eq!(compact_count(42_000), "42k");
        assert_eq!(compact_count(1_250_000), "1.2M");
    }

    #[test]
    fn conditional_probability_handles_zero_denominator() {
        let profile = CooccurrenceProfile::default();

        assert_eq!(profile.conditional_probability("F", "S"), 0.0);
    }
}
