use maud::{Render, html};
use pretty_bytes_typed::pretty_bytes;

const GRAPH_Y_LINES: u32 = 11;
const GRAPH_X_LINES: u32 = 20;
const LINE_SPACING: u32 = 10;

pub struct GraphSeries {
    points: Vec<(u32, f32)>,
    color: String,
}

#[derive(Clone, Copy)]
pub enum Axis {
    Percent,
    Temp,
    Bytes,
}

impl Axis {
    fn get_labels(self) -> [String; GRAPH_Y_LINES as usize] {
        let generator_fn = match self {
            Self::Percent => |x| format!("{}%", 10 * x),
            Self::Temp => |x| format!("{}ºC", 10 * x + 20),
            Self::Bytes => |x| pretty_bytes(10_u64.pow(x as u32), None).to_string(),
        };

        std::array::from_fn(generator_fn)
    }

    // Translates data from [min, max] to [0, 100]
    // Percent: [0, 100]
    // Temp: [20, 120]
    // Bytes: [1, 10^10] (log)
    fn interpolate(self, data: f32) -> f32 {
        match self {
            Self::Percent => data,
            Self::Temp => data - 20.,
            Self::Bytes => 10. * data.log10(),
        }
    }
}

pub struct SvgGraph {
    series: Vec<GraphSeries>,
    axis: Axis,
}

impl SvgGraph {
    pub fn new(axis: Axis) -> Self {
        Self {
            series: Vec::new(),
            axis,
        }
    }

    pub fn add_series(&mut self, points: impl Iterator<Item = f32>, color: &str) {
        let points = points.map(|x| self.axis.interpolate(x));

        // Creates (x, y) pairs starting from the right
        let points: Vec<_> = (0..GRAPH_X_LINES).rev().zip(points).collect();

        let series = GraphSeries {
            points,
            color: color.to_string(),
        };

        self.series.push(series);
    }
}

impl Render for SvgGraph {
    fn render(&self) -> maud::Markup {
        let left_margin = LINE_SPACING * 5 / 3;
        let right_margin = LINE_SPACING / 2;
        let top_margin = LINE_SPACING / 2;
        let bottom_margin = LINE_SPACING / 2;

        let graph_width = LINE_SPACING * (GRAPH_X_LINES - 1);
        let graph_height = LINE_SPACING * (GRAPH_Y_LINES - 1);

        let x_end = left_margin + graph_width;
        let y_end = top_margin + graph_height;

        let total_width = left_margin + graph_width + right_margin;
        let total_height = top_margin + graph_height + bottom_margin;

        let view_box = format!("0 0 {total_width} {total_height}");

        let h_lines = (0..GRAPH_X_LINES).map(|x| left_margin + LINE_SPACING * x);
        let v_lines = (0..GRAPH_Y_LINES).map(|y| top_margin + LINE_SPACING * y);

        let axis_ys = (0..GRAPH_Y_LINES)
            .rev()
            .map(|y| LINE_SPACING * y + top_margin + 1);
        let axis = axis_ys.zip(self.axis.get_labels());

        html! {
            svg .graph viewBox=(view_box) {
                @for x in h_lines {
                    line x1=(x) y1=(top_margin) x2=(x) y2=(y_end) {}
                }
                @for (y, val) in axis {
                    text x="1" y=(y) { (val) }
                }
                @for y in v_lines {
                    line x1=(left_margin) y1=(y) x2=(x_end) y2=(y) {}
                }
                @for series in &self.series {
                    @let points = series.points.iter().map(|(x, y)| {
                        (left_margin + (LINE_SPACING * x), y_end as f32 - y)
                    });
                    @let polyline_points = {
                        use core::fmt::Write;

                        let mut acc = String::new();
                        for (x, y) in points.clone() {
                            let _ = write!(acc, "{x},{y} ");
                        }
                        acc
                    };
                    g fill=(&series.color) {
                        @for (x, y) in points {
                            circle cx=(x) cy=(y) r="1.5" {}
                        }
                    }
                    polyline points=(polyline_points) stroke=(&series.color) fill="none" {}
                }
            }
        }
    }
}
