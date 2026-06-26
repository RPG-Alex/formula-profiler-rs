use std::{cmp::Ordering, path::Path};

use plotters::prelude::*;

use crate::{
    error::Result, population::PopulationSummaryRow, reports::ReportPaths,
    visuals::common::figure_error,
};

#[derive(Debug, Copy, Clone)]
pub enum PopulationMetric {
    TargetCount,
    PercentTargetWithinGroup,
}

impl PopulationMetric {
    fn value(self, row: &PopulationSummaryRow) -> f64 {
        match self {
            Self::TargetCount => row.target_count as f64,
            Self::PercentTargetWithinGroup => row.percent_target_within_group,
        }
    }

    fn axis_label(self) -> &'static str {
        match self {
            Self::TargetCount => "Target-positive records",
            Self::PercentTargetWithinGroup => "Percent target-positive within group",
        }
    }

    fn file_suffix(self) -> &'static str {
        match self {
            Self::TargetCount => "target_count",
            Self::PercentTargetWithinGroup => "percent_target",
        }
    }
}

pub fn write_standard_population_figures(
    reports: &ReportPaths,
    stem: &str,
    title_root: &str,
    rows: &[PopulationSummaryRow],
) -> Result<()> {
    render_top_population_chart(
        reports
            .figure(&format!("top_{stem}_by_{}.svg", PopulationMetric::TargetCount.file_suffix())),
        &format!("{title_root}: Top groups by target count"),
        rows,
        PopulationMetric::TargetCount,
        15,
        0,
    )?;

    render_top_population_chart(
        reports.figure(&format!(
            "top_{stem}_by_{}.svg",
            PopulationMetric::PercentTargetWithinGroup.file_suffix()
        )),
        &format!("{title_root}: Top groups by percent target"),
        rows,
        PopulationMetric::PercentTargetWithinGroup,
        15,
        30,
    )?;

    Ok(())
}

fn render_top_population_chart(
    path: impl AsRef<Path>,
    title: &str,
    rows: &[PopulationSummaryRow],
    metric: PopulationMetric,
    max_items: usize,
    min_total_count: usize,
) -> Result<()> {
    let mut filtered: Vec<PopulationSummaryRow> = rows
        .iter()
        .filter(|row| row.total_count >= min_total_count)
        .filter(|row| !row.value.starts_with("TOTAL_"))
        .cloned()
        .collect();

    filtered
        .sort_by(|a, b| metric.value(b).partial_cmp(&metric.value(a)).unwrap_or(Ordering::Equal));

    filtered.truncate(max_items);
    filtered.reverse();

    if filtered.is_empty() {
        return Ok(());
    }

    let max_value = filtered.iter().map(|row| metric.value(row)).fold(0.0_f64, f64::max).max(1.0);

    let root = SVGBackend::new(path.as_ref(), (1400, 900)).into_drawing_area();
    root.fill(&WHITE).map_err(figure_error)?;

    let y_count = filtered.len() as i32;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
        .margin(25)
        .x_label_area_size(60)
        .y_label_area_size(280)
        .build_cartesian_2d(0f64..(max_value * 1.15), 0i32..y_count)
        .map_err(figure_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc(metric.axis_label())
        .y_labels(filtered.len())
        .y_label_style(("sans-serif", 18))
        .y_label_formatter(&|y| {
            usize::try_from(*y)
                .ok()
                .and_then(|index| filtered.get(index))
                .map(|row| row.value.clone())
                .unwrap_or_default()
        })
        .draw()
        .map_err(figure_error)?;

    for (idx, row) in filtered.iter().enumerate() {
        let y0 = idx as i32;
        let y1 = y0 + 1;
        let value = metric.value(row);

        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(0.0, y0), (value, y1)],
                BLUE.mix(0.6).filled(),
            )))
            .map_err(figure_error)?;

        let label = match metric {
            PopulationMetric::TargetCount => {
                format!("{} ({:.2}%)", row.target_count, row.percent_target_within_group)
            }
            PopulationMetric::PercentTargetWithinGroup => {
                format!("{:.2}% (n={})", row.percent_target_within_group, row.target_count)
            }
        };

        chart
            .draw_series(std::iter::once(Text::new(
                label,
                (value + max_value * 0.01, y0),
                ("sans-serif", 16).into_font(),
            )))
            .map_err(figure_error)?;
    }

    root.present().map_err(figure_error)?;

    Ok(())
}
