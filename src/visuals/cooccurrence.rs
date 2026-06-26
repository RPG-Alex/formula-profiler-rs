use std::path::Path;

use plotters::prelude::*;

use crate::{
    cooccurrence::CooccurrenceProfile,
    error::Result,
    visuals::common::{compact_count, figure_error},
};

pub fn write_raw_count_heatmap(
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

pub fn write_conditional_probability_heatmap(
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

fn heatmap_font_size(cell_size: i32, scale: f64, min: i32, max: i32) -> i32 {
    ((cell_size as f64 * scale).round() as i32).clamp(min, max)
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
