use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use plotters::{coord::Shift, prelude::*};

use crate::{
    error::Result,
    reports::ReportPaths,
    visuals::common::{compact_count, figure_error, percent},
};

pub(crate) fn write_atom_count_distribution_figure(
    reports: &ReportPaths,
    target_element: &str,
    total_records: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    render_atom_count_distribution_chart(
        reports.figure("target_atom_count_distribution.svg"),
        target_element,
        total_records,
        distribution,
    )
}

fn render_atom_count_distribution_chart(
    path: impl AsRef<Path>,
    target_element: &str,
    total_records: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    if distribution.is_empty() {
        return Ok(());
    }

    let root = SVGBackend::new(path.as_ref(), (1200, 800)).into_drawing_area();

    root.fill(&WHITE).map_err(figure_error)?;

    draw_atom_count_chart(&root, target_element, total_records, distribution)?;

    root.present().map_err(figure_error)?;

    Ok(())
}

fn draw_atom_count_chart(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    target_element: &str,
    total_records: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    let max_count = distribution.values().copied().max().unwrap_or(1).max(1) as f64;

    let max_atom_count = distribution.keys().copied().max().unwrap_or(0) as i32;

    let mut chart = ChartBuilder::on(root)
        .caption(format!("{target_element} atom-count distribution"), ("sans-serif", 32))
        .margin(30)
        .x_label_area_size(60)
        .y_label_area_size(90)
        .build_cartesian_2d(0i32..(max_atom_count + 1), 0f64..(max_count * 1.15))
        .map_err(figure_error)?;

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc(format!("Number of {target_element} atoms in formula"))
        .y_desc("Profiled records")
        .x_labels((max_atom_count as usize + 1).min(20))
        .y_label_formatter(&|value| compact_count(*value as usize))
        .draw()
        .map_err(figure_error)?;

    chart
        .draw_series(distribution.iter().map(|(atom_count, record_count)| {
            let x0 = *atom_count as i32;
            let x1 = x0 + 1;
            let y = *record_count as f64;

            Rectangle::new([(x0, 0.0), (x1, y)], BLUE.mix(0.6).filled())
        }))
        .map_err(figure_error)?;

    let labeled_atom_counts = top_labeled_atom_counts(distribution, 15);

    chart
        .draw_series(
            distribution
                .iter()
                .filter(|(atom_count, _)| labeled_atom_counts.contains(atom_count))
                .map(|(atom_count, record_count)| {
                    let percentage = percent(*record_count, total_records);

                    Text::new(
                        format!("{} ({percentage:.1}%)", compact_count(*record_count)),
                        (*atom_count as i32, *record_count as f64 + max_count * 0.015),
                        ("sans-serif", 16).into_font(),
                    )
                }),
        )
        .map_err(figure_error)?;

    Ok(())
}

fn top_labeled_atom_counts(distribution: &BTreeMap<usize, usize>, limit: usize) -> BTreeSet<usize> {
    let mut counts = distribution
        .iter()
        .map(|(atom_count, record_count)| (*atom_count, *record_count))
        .collect::<Vec<_>>();

    counts.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    counts.into_iter().take(limit).map(|(atom_count, _)| atom_count).collect()
}
