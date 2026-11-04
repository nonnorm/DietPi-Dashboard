use maud::{Render, html};
use pretty_bytes_typed::pretty_bytes;

use crate::pages::template::Icon;

const GRAPH_Y_LINES: u32 = 11;
const GRAPH_X_LINES: u32 = 20;
const LINE_SPACING: u32 = 10;

pub struct GraphSeries {
    points: Vec<(u32, f32)>,
    color: &'static str,
    label: &'static str,
    i18n_key: &'static str,
}

#[derive(Clone, Copy)]
pub enum Axis {
    Percent,
    Temp,
    Bytes,
}

impl Axis {
    fn format_val(self, val: f32) -> String {
        match self {
            Self::Percent => format!("{val:.1}%"),
            Self::Temp => format!("{val:.1}ºC"),
            Self::Bytes => pretty_bytes(val as u64, Some(0)).to_string(),
        }
    }

    fn format_tick(self, val: f32) -> String {
        match self {
            Self::Percent => format!("{val:.0}%"),
            Self::Temp => format!("{val:.0}º"),
            Self::Bytes => pretty_bytes(val as u64, Some(0)).to_string(),
        }
    }

    fn get_labels(self) -> [String; GRAPH_Y_LINES as usize] {
        let generator_fn = |x| {
            self.format_tick(match self {
                Self::Percent => (10 * x) as f32,
                Self::Temp => (10 * x + 20) as f32,
                Self::Bytes => 10_u64.pow(x as u32) as f32,
            })
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

    pub fn add_series(
        &mut self,
        points: impl Iterator<Item = f32>,
        color: &'static str,
        label: &'static str,
        i18n_key: &'static str,
    ) {
        // Creates (x, y) pairs starting from the right
        let points: Vec<_> = (0..GRAPH_X_LINES).rev().zip(points).collect();

        let series = GraphSeries {
            points,
            color,
            label,
            i18n_key,
        };

        self.series.push(series);
    }
}

impl Render for SvgGraph {
    fn render(&self) -> maud::Markup {
        let left_margin = LINE_SPACING * 5;
        let right_margin = LINE_SPACING;
        let top_margin = LINE_SPACING;
        let bottom_margin = LINE_SPACING;

        let graph_width = LINE_SPACING * (GRAPH_X_LINES - 1);
        let graph_height = LINE_SPACING * (GRAPH_Y_LINES - 1);

        let x_end = left_margin + graph_width;
        let y_end = top_margin + graph_height;

        let total_width = left_margin + graph_width + right_margin;
        let total_height = top_margin + graph_height + bottom_margin;

        let axis = self.axis.get_labels().into_iter().enumerate();

        html! {
            div .graph-wrapper {
                svg .graph viewBox={"0 0 " (total_width) " " (total_height)} {
                    @for (i, label) in axis {
                        @let y = y_end - (i as u32) * LINE_SPACING;
                        line x1=(left_margin) y1=(y) x2=(x_end) y2=(y) {}
                        text x=(left_margin - 3) y=(y) text-anchor="end" dominant-baseline="middle" { (label) }
                    }
                    @for i in 0..GRAPH_X_LINES {
                        @let x = left_margin + i * LINE_SPACING;
                        line x1=(x) y1=(top_margin) x2=(x) y2=(y_end) {}
                    }
                    @for series in &self.series {
                        @let points = series.points.iter().map(|&(x, y)| {
                            let y = self.axis.interpolate(y);
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
                        polyline
                            points=(polyline_points)
                            stroke=(&series.color)
                            fill="none"
                        {}
                        rect width=(graph_width) height=(graph_height) x=(left_margin) y=(top_margin) fill="transparent"
                            nm-bind={"
                                onmousemove: (e) => {
                                    x = e.offsetX;
                                    const { x: rectX, width } = this.getBoundingClientRect();
                                    idx = Math.round(19-(e.clientX - rectX)/width*19);
                                },
                                onmouseleave: () => (x = null, idx = 0)
                            "}
                        {}
                    }
                }
                div .ui-line nm-bind="'style.left': () => (x ?? 9999) + 'px'" {}
            }
            div .legend {
                @for series in &self.series {
                    @let point_vals = {
                        use core::fmt::Write;

                        let mut acc = String::new();
                        for &(_, y) in &series.points {
                            let val = self.axis.format_val(y);

                            let _ = write!(acc, "'{val}',");
                        }
                        acc
                    };
                    p {
                        span style={"color:"(series.color)} { (Icon::new("fa6-solid-square").size(16)) }
                        span data-i18n=(series.i18n_key) { (series.label) }
                        span nm-bind={"textContent: () => ["(point_vals)"][idx]"} {}
                    }
                }
            }
        }
    }
}
