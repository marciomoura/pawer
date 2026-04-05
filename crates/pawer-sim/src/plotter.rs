use plotters::prelude::*;

use crate::logger::Logger;

/// Generate an SVG plot of the requested signals.
///
/// Each signal is drawn as a separate line series with automatic color
/// assignment. The plot includes a legend, axis labels, and grid.
pub fn plot_signals(
    logger: &Logger,
    signal_names: &[String],
    output_path: &str,
) -> Result<(), PlotError> {
    if logger.is_empty() {
        return Err(PlotError("No data to plot. Run /simulate first.".into()));
    }

    // Collect all series data and compute axis ranges
    let mut all_series: Vec<(String, Vec<(f64, f64)>)> = Vec::new();
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut t_min = f64::INFINITY;
    let mut t_max = f64::NEG_INFINITY;

    for name in signal_names {
        let series = logger.series(name);
        if series.is_empty() {
            return Err(PlotError(format!(
                "Signal \"{}\" has no data. Check /signals for available names.",
                name
            )));
        }
        for &(t, v) in &series {
            let v = v as f64;
            y_min = y_min.min(v);
            y_max = y_max.max(v);
            t_min = t_min.min(t);
            t_max = t_max.max(t);
        }
        let series_f64: Vec<(f64, f64)> = series.into_iter().map(|(t, v)| (t, v as f64)).collect();
        all_series.push((name.clone(), series_f64));
    }

    // Add some margin to y-axis
    let y_range = y_max - y_min;
    if y_range < 1e-12 {
        y_min -= 0.5;
        y_max += 0.5;
    } else {
        let margin = y_range * 0.08;
        y_min -= margin;
        y_max += margin;
    }

    let root = SVGBackend::new(output_path, (960, 540)).into_drawing_area();
    root.fill(&WHITE)
        .map_err(|e| PlotError(format!("Drawing error: {}", e)))?;

    let mut chart = ChartBuilder::on(&root)
        .caption("pawer-sim", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| PlotError(format!("Chart build error: {}", e)))?;

    chart
        .configure_mesh()
        .x_desc("Time (s)")
        .y_desc("Value")
        .draw()
        .map_err(|e| PlotError(format!("Mesh draw error: {}", e)))?;

    let palette = [
        &RGBColor(0x1f, 0x77, 0xb4), // blue
        &RGBColor(0xff, 0x7f, 0x0e), // orange
        &RGBColor(0x2c, 0xa0, 0x2c), // green
        &RGBColor(0xd6, 0x27, 0x28), // red
        &RGBColor(0x94, 0x67, 0xbd), // purple
        &RGBColor(0x8c, 0x56, 0x4b), // brown
        &RGBColor(0xe3, 0x77, 0xc2), // pink
        &RGBColor(0x7f, 0x7f, 0x7f), // gray
    ];

    for (i, (name, data)) in all_series.iter().enumerate() {
        let color = palette[i % palette.len()];
        chart
            .draw_series(LineSeries::new(data.iter().copied(), color.stroke_width(2)))
            .map_err(|e| PlotError(format!("Series draw error: {}", e)))?
            .label(name.as_str())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .map_err(|e| PlotError(format!("Legend draw error: {}", e)))?;

    root.present()
        .map_err(|e| PlotError(format!("SVG render error: {}", e)))?;

    Ok(())
}

#[derive(Debug)]
pub struct PlotError(pub String);

impl std::fmt::Display for PlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
