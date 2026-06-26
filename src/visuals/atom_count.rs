use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use plotters::prelude::*;

use crate::{
    error::Result,
    reports::ReportPaths,
    visuals::common::{compact_count, figure_error, percent},
};

pub fn write_atom_count_distribution_figure(
    reports: &ReportPaths,
    target_element: &str,
    records_with_formula: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    render_atom_count_distribution_chart(
        reports.figure("target_atom_count_distribution.svg"),
        target_element,
        records_with_formula,
        distribution,
    )
}

fn render_atom_count_distribution_chart(
    path: impl AsRef<Path>,
    target_element: &str,
    records_with_formula: usize,
    distribution: &BTreeMap<usize, usize>,
) -> Result<()> {
    if distribution.is_empty() {
        return Ok(());
    }

    let max_count = distribution.values().copied().max().unwrap_or(1).max(1) as f64;
    let max_atom_count = distribution.keys().copied().max().unwrap_or(0) as i32;

    let root = SVGBackend::new(path.as_ref(), (1200, 800)).into_drawing_area();
    root.fill(&WHITE).map_err(figure_error)?;

    let mut chart = ChartBuilder::on(&root)
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
        .y_desc("Formula-bearing records")
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

    let mut labeled_atom_counts = distribution
        .iter()
        .map(|(atom_count, record_count)| (*atom_count, *record_count))
        .collect::<Vec<_>>();

    labeled_atom_counts
        .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let labeled_atom_counts = labeled_atom_counts
        .into_iter()
        .take(15)
        .map(|(atom_count, _)| atom_count)
        .collect::<BTreeSet<_>>();

    for (atom_count, record_count) in distribution {
        if !labeled_atom_counts.contains(atom_count) {
            continue;
        }

        let percent = percent(*record_count, records_with_formula);

        chart
            .draw_series(std::iter::once(Text::new(
                format!("{} ({:.1}%)", compact_count(*record_count), percent),
                (*atom_count as i32, *record_count as f64 + max_count * 0.015),
                ("sans-serif", 16).into_font(),
            )))
            .map_err(figure_error)?;
    }

    root.present().map_err(figure_error)?;

    Ok(())
}
