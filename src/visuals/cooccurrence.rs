use std::path::Path;

use plotters::{
    coord::Shift,
    prelude::*,
    style::text_anchor::{HPos, Pos, VPos},
};

use crate::{
    error::Result,
    profiler::DatasetProfile,
    visuals::common::{compact_count, figure_error},
};

struct HeatmapLayout {
    cell_size: i32,
    label_font_size: i32,
    value_font_size: i32,
    left_margin: i32,
    top_margin: i32,
    right_margin: i32,
    bottom_margin: i32,
}

impl HeatmapLayout {
    fn new(element_count: usize) -> Self {
        let cell_size = heatmap_cell_size(element_count);
        let label_font_size = heatmap_font_size(cell_size, 0.35, 8, 22);
        let value_font_size = heatmap_font_size(cell_size, 0.28, 6, 18);

        Self {
            cell_size,
            label_font_size,
            value_font_size,
            left_margin: 120,
            top_margin: 80 + label_font_size * 3,
            right_margin: 60,
            bottom_margin: 50,
        }
    }

    fn heatmap_width(&self, element_count: usize) -> i32 {
        self.cell_size * element_count as i32
    }

    fn heatmap_height(&self, element_count: usize) -> i32 {
        self.cell_size * element_count as i32
    }

    fn figure_width(&self, element_count: usize) -> u32 {
        (self.left_margin + self.right_margin + self.heatmap_width(element_count)) as u32
    }

    fn figure_height(&self, element_count: usize) -> u32 {
        (self.top_margin + self.bottom_margin + self.heatmap_height(element_count)) as u32
    }

    fn heatmap_center_x(&self, element_count: usize) -> i32 {
        self.left_margin + self.heatmap_width(element_count) / 2
    }
}

pub(crate) fn write_raw_count_heatmap(
    path: impl AsRef<Path>,
    profile: &DatasetProfile,
    elements: &[String],
) -> Result<()> {
    let max_log_value = elements
        .iter()
        .flat_map(|row| elements.iter().map(move |column| profile.pair_count(row, column) as f64))
        .map(|value| (value + 1.0).log10())
        .fold(0.0_f64, f64::max)
        .max(1.0);

    render_heatmap(
        path,
        "Element co-occurrence counts",
        elements,
        |row, column| {
            let count = profile.pair_count(row, column);
            let scaled = ((count as f64 + 1.0).log10() / max_log_value).clamp(0.0, 1.0);

            (scaled, compact_count(count))
        },
        heatmap_color,
    )
}

pub(crate) fn write_conditional_probability_heatmap(
    path: impl AsRef<Path>,
    profile: &DatasetProfile,
    elements: &[String],
) -> Result<()> {
    render_heatmap(
        path,
        "P(column element | row element)",
        elements,
        |row, column| {
            let probability = profile.conditional_probability(row, column);

            (probability.clamp(0.0, 1.0), format!("{:.0}%", probability * 100.0))
        },
        heatmap_color,
    )
}

pub(crate) fn write_normalized_pmi_heatmap(
    path: impl AsRef<Path>,
    profile: &DatasetProfile,
    elements: &[String],
) -> Result<()> {
    render_heatmap(
        path,
        "Normalized element association (NPMI)",
        elements,
        |row, column| {
            let Some(npmi) = profile.normalized_pmi(row, column) else {
                return (0.5, "—".to_string());
            };
            // converts NPMI from -1 to 1 to 0-1
            let scaled = ((npmi + 1.0) / 2.0).clamp(0.0, 1.0);
            (scaled, format!("{npmi:.2}"))
        },
        association_heatmap_color,
    )
}

fn association_heatmap_color(value: f64) -> RGBColor {
    let value = value.clamp(0.0, 1.0);

    if value < 0.5 {
        let intensity = value * 2.0;

        RGBColor((255.0 * intensity) as u8, (255.0 * intensity) as u8, 255)
    } else {
        let intensity = (1.0 - value) * 2.0;
        RGBColor(255, (255.0 * intensity) as u8, (255.0 * intensity) as u8)
    }
}

fn render_heatmap<F, C>(
    path: impl AsRef<Path>,
    title: &str,
    elements: &[String],
    value_for: F,
    color_for: C,
) -> Result<()>
where
    F: Fn(&str, &str) -> (f64, String),
    C: Fn(f64) -> RGBColor,
{
    if elements.is_empty() {
        return Ok(());
    }

    let layout = HeatmapLayout::new(elements.len());

    let root = SVGBackend::new(
        path.as_ref(),
        (layout.figure_width(elements.len()), layout.figure_height(elements.len())),
    )
    .into_drawing_area();

    root.fill(&WHITE).map_err(figure_error)?;

    render_title(title, elements.len(), &layout, &root)?;
    render_axis_labels(elements, &layout, &root)?;
    render_cells(elements, &layout, value_for, color_for, &root)?;

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

fn render_axis_labels(
    elements: &[String],
    layout: &HeatmapLayout,
    root: &DrawingArea<SVGBackend<'_>, Shift>,
) -> Result<()> {
    let column_style = TextStyle::from(("sans-serif", layout.label_font_size).into_font())
        .pos(Pos::new(HPos::Center, VPos::Bottom));

    let row_style = TextStyle::from(("sans-serif", layout.label_font_size).into_font())
        .pos(Pos::new(HPos::Right, VPos::Center));

    for (index, element) in elements.iter().enumerate() {
        let index = index as i32;

        let x = layout.left_margin + index * layout.cell_size + layout.cell_size / 2;
        let y = layout.top_margin + index * layout.cell_size + layout.cell_size / 2;

        root.draw(&Text::new(element.as_str(), (x, layout.top_margin - 18), column_style.clone()))
            .map_err(figure_error)?;

        root.draw(&Text::new(element.as_str(), (layout.left_margin - 18, y), row_style.clone()))
            .map_err(figure_error)?;
    }

    Ok(())
}

fn render_cells<F, C>(
    elements: &[String],
    layout: &HeatmapLayout,
    value_for: F,
    color_for: C,
    root: &DrawingArea<SVGBackend<'_>, Shift>,
) -> Result<()>
where
    F: Fn(&str, &str) -> (f64, String),
    C: Fn(f64) -> RGBColor,
{
    for (row_index, row_element) in elements.iter().enumerate() {
        for (column_index, column_element) in elements.iter().enumerate() {
            let x0 = layout.left_margin + column_index as i32 * layout.cell_size;
            let y0 = layout.top_margin + row_index as i32 * layout.cell_size;

            render_cell(
                (x0, y0),
                layout,
                value_for(row_element, column_element),
                &color_for,
                root,
            )?;
        }
    }

    Ok(())
}

fn render_cell<C>(
    position: (i32, i32),
    layout: &HeatmapLayout,
    value: (f64, String),
    color_for: &C,
    root: &DrawingArea<SVGBackend<'_>, Shift>,
) -> Result<()>
where
    C: Fn(f64) -> RGBColor,
{
    let (x0, y0) = position;
    let x1 = x0 + layout.cell_size;
    let y1 = y0 + layout.cell_size;

    let (scaled_value, label) = value;
    let color = color_for(scaled_value);

    root.draw(&Rectangle::new([(x0, y0), (x1, y1)], color.filled())).map_err(figure_error)?;

    root.draw(&Rectangle::new(
        [(x0, y0), (x1, y1)],
        ShapeStyle::from(&WHITE.mix(0.85)).stroke_width(1),
    ))
    .map_err(figure_error)?;

    let text_color = heatmap_text_color(color);

    let text_style = TextStyle::from(("sans-serif", layout.value_font_size).into_font())
        .color(&text_color)
        .pos(Pos::new(HPos::Center, VPos::Center));

    root.draw(&Text::new(
        label,
        (x0 + layout.cell_size / 2, y0 + layout.cell_size / 2),
        text_style,
    ))
    .map_err(figure_error)?;

    Ok(())
}

fn heatmap_text_color(color: RGBColor) -> RGBColor {
    let luminance =
        0.299 * f64::from(color.0) + 0.587 * f64::from(color.1) + 0.114 * f64::from(color.2);

    if luminance < 150.0 { WHITE } else { BLACK }
}

fn render_title(
    title: &str,
    element_count: usize,
    layout: &HeatmapLayout,
    root: &DrawingArea<SVGBackend<'_>, Shift>,
) -> Result<()> {
    let title_style =
        TextStyle::from(("sans-serif", 34).into_font()).pos(Pos::new(HPos::Center, VPos::Center));

    root.draw(&Text::new(title, (layout.heatmap_center_x(element_count), 42), title_style))
        .map_err(figure_error)?;

    Ok(())
}
