#![allow(unused)]

use std::collections::HashMap;

use crate::svg::{self, opts, Params, Tag};

/// The basic plotting primitive. Can be converted into a Vec of `svg::Tag`.
trait Element {
    /// Convert `Element` into a Vec of `svg::Tag`.
    ///
    /// All elements are placed relative to an axis which itself is sized and
    /// placed relative to a figure. For correct placement, the axis and figure
    /// thus have to be passed into the function.
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>>;
    fn add_svg_property(&mut self, key: &str, value: &str);
    fn identifier(&self) -> &str;
}

// ----------------------------------------------------------------------------
//
//
// Figure
//
//
// ----------------------------------------------------------------------------

/// The Figure defines the overall size of a plot and holds the axes and
/// optional annotations.
pub struct Figure {
    width: u64,
    height: u64,
    axes: Vec<Axis>,
    annotations: Vec<Box<dyn Element>>,
}

impl Figure {
    pub fn empty(width: u64, height: u64) -> Self {
        Self {
            width,
            height,
            axes: Vec::new(),
            annotations: Vec::new(),
        }
    }

    pub fn new(width: u64, height: u64) -> Self {
        let ax = Axis::default();
        Self {
            width,
            height,
            axes: vec![ax],
            annotations: Vec::new(),
        }
    }

    pub fn add_axis(&mut self, ax: Axis) {
        self.axes.push(ax);
    }

    /// Render this `Figure` to raw SVG markup.
    pub fn render(&self) -> String {
        let mut root = Tag::<svg::SVG>::new(self.width, self.height, None);
        for ax in self.axes.iter() {
            let ax_tags = ax.to_tags(&self);
            root.add_children(ax_tags);
        }
        crate::svg::render(&root)
    }
}

impl Default for Figure {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

// ----------------------------------------------------------------------------
//
//
// Axis
//
//
// ----------------------------------------------------------------------------

/// The container for plots and other elements.
pub struct Axis {
    /// u coordinate for placement in Figure, normalized to [0, 1]
    u: f64,
    /// v coordinate for placement in Figure, normalized to [0, 1]
    v: f64,
    width: f64,
    height: f64,
    limits: [f64; 4],
    ticks: Ticks,
    elements: Vec<Box<dyn Element>>,
    properties: svg::Params,
}

impl Axis {
    pub fn new(u: f64, v: f64, width: f64, height: f64) -> Self {
        Axis {
            u,
            v,
            width,
            height,
            limits: [0.0, 1.0, 0.0, 1.0],
            ticks: Default::default(),
            elements: Vec::new(),
            properties: element_opts(&[("fill", "white"), ("stroke", "black")]),
        }
    }

    pub fn xlim(&mut self, xmin: f64, xmax: f64) {
        self.limits[0] = xmin;
        self.limits[1] = xmax;
    }

    pub fn ylim(&mut self, ymin: f64, ymax: f64) {
        self.limits[2] = ymin;
        self.limits[3] = ymax;
    }

    pub fn add_line(&mut self, line: LinePlot) {
        self.elements.push(Box::new(line));
    }

    pub fn add_label(&mut self, label: Text) {
        self.elements.push(Box::new(label));
    }

    pub fn with_xlim(mut self, xmin: f64, xmax: f64) -> Self {
        self.xlim(xmin, xmax);
        self.autoticks();
        self
    }

    pub fn with_xlabel(mut self, text: &str) -> Self {
        let xlabel = Text {
            text: text.to_owned(),
            u: 0.5,
            v: -0.1,
            angle: 0.0,
            properties: element_opts(&[("text-anchor", "middle")]),
        };
        self.add_label(xlabel);
        self
    }

    pub fn with_ylim(mut self, ymin: f64, ymax: f64) -> Self {
        self.ylim(ymin, ymax);
        self.autoticks();
        self
    }

    pub fn with_ylabel(mut self, text: &str) -> Self {
        let ylabel = Text {
            text: text.to_owned(),
            u: -0.15,
            v: 0.5,
            angle: 270.0,
            properties: element_opts(&[("text-anchor", "middle")]),
        };
        self.add_label(ylabel);
        self
    }

    pub fn with_lineplot(mut self, line: LinePlot) -> Self {
        self.elements.push(Box::new(line));
        self
    }

    pub fn insert_into(self, fig: &mut Figure) {
        fig.add_axis(self);
    }

    fn autoticks(&mut self) {
        let positions = |min: f64, max: f64, mult: f64| -> Vec<f64> {
            let span_mag = (max - min).log10().floor() as i32;
            let f = 10.0f64.powi(span_mag - 1);
            let mut step = f * mult;
            while (max - min) / step <= 3.0 {
                step /= 2.0;
            }
            while (max - min) / step > 10.0 {
                step *= 2.0;
            }

            let mut x = (min / f).ceil() * f;
            let mut pos = Vec::new();
            while x < max {
                pos.push(x);
                x += step;
            }

            pos
        };

        let [xmin, xmax, ymin, ymax] = self.limits_ordered();

        // Choosing a multiplicator for the stepsize is done depending
        // on whether the figure is in portrait or landscape format.
        let (mx, my) = if self.width < self.height {
            (5.0, 2.5)
        } else {
            (2.5, 5.0)
        };

        self.ticks.xpos = positions(xmin, xmax, mx);
        self.ticks.ypos = positions(ymin, ymax, my);
    }

    fn transformations(
        &self,
        fig: &Figure,
    ) -> (
        impl Fn(f64) -> f64,
        impl Fn(f64) -> f64,
        impl Fn(f64) -> f64,
        impl Fn(f64) -> f64,
    ) {
        let (fw, fh) = (fig.width, fig.height);
        let (au, av, aw, ah) = (self.u, self.v, self.width, self.height);
        let [xmin, xmax, ymin, ymax] = self.limits;

        let x = move |u| fw as f64 * (au + u * aw);
        let y = move |v| fh as f64 * (av + v * ah);
        let u = move |x| (x - xmin) / (xmax - xmin);
        let v = move |y| 1.0 - (y - ymin) / (ymax - ymin);

        (x, y, u, v)
    }

    /// Convert `Axis` into a Vec of `svg::Tag`.
    ///
    /// Since we only need the figure for placing `Axis`, `Axis` is not an
    /// `Element` and implements this function without using the trait.
    fn to_tags(&self, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (w, h) = (fig.width as f64, fig.height as f64);
        let mut children: Vec<_> = self
            .elements
            .iter()
            .map(|el| el.to_tags(self, fig))
            .reduce(|mut ts, next_ts| {
                ts.extend(next_ts);
                ts
            })
            .unwrap_or(Vec::new());
        children.extend(self.ticks.to_tags(&self, fig));
        let mut ax_rect = Tag::<svg::Rect>::new(
            w * self.u,
            h * self.v,
            self.width * w,
            self.height * h,
            Some(self.properties.clone()),
        );
        ax_rect.add_children(children);
        vec![Box::new(ax_rect)]
    }

    fn limits_ordered(&self) -> [f64; 4] {
        let [mut xmin, mut xmax, mut ymin, mut ymax] = self.limits;
        // Bring limits in correct order.
        let (xmin, xmax) = (xmin.min(xmax), xmin.max(xmax));
        let (ymin, ymax) = (ymin.min(ymax), ymin.max(ymax));
        [xmin, xmax, ymin, ymax]
    }

    /// This function checks the xy-data in LinePlot for going out of the axis
    /// boundaries. If the data goes out of the axis, a new datapoint is added
    /// (at the crossing of the axis). Furthermore, the data is subdivided into
    /// segments at each axis crossing, so it can be rendered with several svg
    /// polyline elements.
    fn segment_lineplot_data(&self, line: &LinePlot) -> Vec<(Vec<f64>, Vec<f64>)> {
        let (xs, ys) = (&line.xs[..], &line.ys[..]);
        let (nx, ny) = (xs.len(), ys.len());
        let n = nx.min(ny);
        if n < 2 {
            return Vec::new();
        }

        let [xmin, xmax, ymin, ymax] = self.limits_ordered();

        // Short circuit if everything fits into axis.
        if xs.iter().all(|x| xmin <= *x && *x <= xmax)
            && ys.iter().all(|y| ymin <= *y && *y <= ymax)
        {
            return vec![(line.xs.clone(), line.ys.clone())];
        }

        let xspan = xmax - xmin;
        let yspan = ymax - ymin;

        // Helper function to find point outside of axis, refering to point by index:
        let outside = |i: usize| xs[i] < xmin || xs[i] > xmax || ys[i] < ymin || ys[i] > ymax;

        // Containers to hold data segments:
        let mut segments = vec![(Vec::<f64>::new(), Vec::<f64>::new())];

        // Iterate datapoints and build up segemnts
        for i in 1..(n - 1) {
            // Current datapoints normalized to axis from 0 to 1 in both x and y
            // directions. This normalization makes later checks and calculations
            // of crossings easier.
            let [xm, xn, ym, yn] = {
                let (k, l) = if xs[i] < xs[i + 1] {
                    (i, i + 1)
                } else {
                    (i + 1, i)
                };
                [
                    (xs[k] - xmin) / xspan,
                    (xs[l] - xmin) / xspan,
                    (ys[k] - ymin) / yspan,
                    (ys[l] - ymin) / yspan,
                ]
            };

            // If the datapoints are _both_ to the left, above, right, or under the
            // axis, respectively, there cannot be any crossings and we can continue.
            if (xm < 0.0 && xn < 0.0)
                || (xm > 1.0 && xn > 1.0)
                || (ym < 0.0 && yn < 0.0)
                || (ym > 1.0 && yn > 1.0)
            {
                continue;
            }

            // If the ith point is outside axis and the current segment is not empty,
            // we have to start a new segment of datapoints.
            if outside(i) {
                if !segments.last().unwrap().0.is_empty() {
                    segments.push((Vec::<f64>::new(), Vec::<f64>::new()));
                }
            // The ith point lies within axis, we add it to the current segment.
            } else {
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(xs[i]);
                    yp_cur.push(ys[i]);
                });
                // If (i + 1)th datapoint is also within axis, we do not need to check
                // for crossings ...
                if !outside(i + 1) {
                    // ... but we have to check if it is the last point, because then
                    // we have to add it to the current (and last) segment.
                    if i + 1 == n {
                        segments.last_mut().map(|(xp_cur, yp_cur)| {
                            xp_cur.push(xs[i]);
                            yp_cur.push(ys[i]);
                        });
                        break;
                    }
                    continue;
                }
            }

            // This is an edge case where we cannot use a line function to interpolate
            // positions of crossings.
            if xm == xn {
                // Because of the earlier check, we know that 0 < x < 1.
                if (ym > 1.0 || yn > 1.0) {
                    segments.last_mut().map(|(xp_cur, yp_cur)| {
                        xp_cur.push(xs[i]);
                        yp_cur.push(ymax);
                    });
                }
                if (ym < 0.0 || yn < 0.0) {
                    segments.last_mut().map(|(xp_cur, yp_cur)| {
                        xp_cur.push(xs[i]);
                        yp_cur.push(ymin);
                    });
                }
                // There cannot be any other crossings, so we continue.
                continue;
            }

            // The line function defined by current data points:
            let m = (yn - ym) / (xn - xm); // slope
            let b = ym - xm * m; // offset

            // We keep track of the number of crossings: because there can only be two
            // we can skip further checks if we reach that number.
            let mut num_crossings = 0;

            // This tests whether the line defined by data points crosses left boundary:
            if 0.0 < b && b <= 1.0 && xm < 0.0 {
                num_crossings += 1;
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(xmin);
                    yp_cur.push(b * yspan + ymin);
                });
            }

            // This tests whether the line defined by data points crosses top boundary:
            if 0.0 <= (1.0 - b) / m && (1.0 - b) / m < 1.0 && (yn > 1.0 || ym > 1.0) {
                num_crossings += 1;
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(xspan * (1.0 - b) / m + xmin);
                    yp_cur.push(ymax);
                });
            }

            // This tests whether the line defined by data points crosses right
            // boundary. Note that if we crossed two times, there can be no more
            // crossings, so we can skip this check.
            if num_crossings < 2 && 0.0 < m + b && m + b <= 1.0 && xn > 1.0 {
                num_crossings += 1;
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(xmax);
                    yp_cur.push((m + b) * yspan + ymin);
                });
            }

            // This tests whether the line defined by data points crosses bottom
            // boundary. Note that if we crossed two times, there can be no more
            // crossings, so we can skip this check.
            if num_crossings < 2 && 0.0 <= -b / m && -b / m < 1.0 && (yn < 0.0 || ym < 0.0) {
                num_crossings += 1;
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(-b * xspan / m + xmin);
                    yp_cur.push(ymin);
                });
            }

            // If the last datapoint falls within the axis, we add it to the current (and last)
            // datapoint segment.
            if i == n - 1 && !outside(i + 1) {
                segments.last_mut().map(|(xp_cur, yp_cur)| {
                    xp_cur.push(xs[i + 1]);
                    yp_cur.push(ys[i + 1]);
                });
            }
        }
        segments
    }
}

impl Default for Axis {
    fn default() -> Self {
        Axis::new(0.15, 0.1, 0.75, 0.8)
    }
}

impl From<[f64; 4]> for Axis {
    fn from(pos_and_dims: [f64; 4]) -> Self {
        let [u, v, width, height] = pos_and_dims;
        Self::new(u, v, width, height)
    }
}

// ----------------------------------------------------------------------------
//
//
// Text
//
//
// ----------------------------------------------------------------------------

/// Text element for placing labes in the axis (also axes labels).
pub struct Text {
    text: String,
    u: f64,
    v: f64,
    angle: f64,
    properties: svg::Params,
}

impl Element for Text {
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (x, y, _, _) = ax.transformations(fig);

        let (x, y) = (x(self.u), y(1.0 - self.v));
        let mut params = Vec::new();
        params.extend(self.properties.iter().map(|(a, b)| (&a[..], &b[..])));

        vec![Box::new(Tag::<svg::Text>::new(
            x,
            y,
            self.angle,
            &self.text,
            svg::opts(&params),
        ))]
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    fn identifier(&self) -> &str {
        "Text"
    }
}

// ----------------------------------------------------------------------------
//
//
// Ticks
//
//
// ----------------------------------------------------------------------------

/// The ticks of the axes. Includes ticks and tick labels.
pub struct Ticks {
    xpos: Vec<f64>,
    ypos: Vec<f64>,
    color: String,
    linewidth: f64,
    properties: svg::Params,
}

impl Default for Ticks {
    fn default() -> Self {
        Self {
            xpos: Vec::new(),
            ypos: Vec::new(),
            color: "black".to_string(),
            linewidth: 1.0,
            properties: Params::new(),
        }
    }
}

impl Element for Ticks {
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (x, y, u, v) = ax.transformations(fig);
        let [xmin, xmax, ymin, ymax] = ax.limits_ordered();

        let width_param = format!("{}", self.linewidth);
        let params = [
            ("stroke", &self.color[..]),
            ("stroke-width", &width_param[..]),
        ];
        let params_xtick_label = [("text-anchor", "middle")];
        let params_ytick_label = [("text-anchor", "end")];

        let mut xticks: Vec<Box<dyn svg::RenderTag>> = Vec::new();
        let mut yticks: Vec<Box<dyn svg::RenderTag>> = Vec::new();

        for xi in self.xpos.iter() {
            let lt = Tag::<svg::Line>::new(x(u(*xi)), x(u(*xi)), y(0.99), y(1.01), opts(&params));
            let tt = Tag::<svg::Text>::new(
                x(u(*xi)),
                y(1.05),
                0.0,
                &format!("{xi}"),
                opts(&params_xtick_label.clone()),
            );
            xticks.push(Box::new(lt));
            xticks.push(Box::new(tt));
        }
        for yi in self.ypos.iter() {
            let lt =
                Tag::<svg::Line>::new(x(-0.005), x(0.005), y(v(*yi)), y(v(*yi)), opts(&params));
            let tt = Tag::<svg::Text>::new(
                x(-0.03),
                y(v(*yi) + 0.02),
                0.0,
                &format!("{yi}"),
                opts(&params_ytick_label.clone()),
            );
            yticks.push(Box::new(lt));
            yticks.push(Box::new(tt));
        }
        xticks.extend(yticks);
        xticks
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    fn identifier(&self) -> &str {
        "Ticks"
    }
}

// ----------------------------------------------------------------------------
//
//
// LinePlot
//
//
// ----------------------------------------------------------------------------

#[derive(Clone)]
pub struct LinePlot {
    xs: Vec<f64>,
    ys: Vec<f64>,
    properties: svg::Params,
}

impl LinePlot {
    pub fn new(xs: &[f64], ys: &[f64]) -> Self {
        Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            properties: svg::Params::new(),
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.properties
            .insert("stroke".to_string(), color.to_string());
        self
    }

    pub fn with_linewidth(mut self, linewidth: f64) -> Self {
        self.properties
            .insert("stroke-width".to_string(), format!("{linewidth}"));
        self
    }

    pub fn set_color(&mut self, color: &str) {
        self.properties
            .insert("stroke".to_string(), color.to_string());
    }

    pub fn set_linewidth(&mut self, linewidth: f64) {
        self.properties
            .insert("stroke-width".to_string(), format!("{linewidth}"));
    }

    pub fn insert_into(self, mut ax: Axis) -> Axis {
        ax.add_line(self);
        ax
    }
}

impl Element for LinePlot {
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (x, y, u, v) = ax.transformations(fig);
        let mut plines: Vec<Box<dyn svg::RenderTag>> = Vec::new();
        for (xs, ys) in ax.segment_lineplot_data(self).into_iter() {
            let xs = xs.iter().map(|xi| x(u(*xi)));
            let ys = ys.iter().map(|yi| y(v(*yi)));
            plines.push(Box::new(Tag::<svg::Polyline>::new(
                xs,
                ys,
                Some(self.properties.clone()),
            )));
        }

        plines
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.properties.insert(key.to_string(), value.to_string());
    }

    fn identifier(&self) -> &str {
        "LinePlot"
    }
}

// ----------------------------------------------------------------------------
//
//
// Helpers
//
//
// ----------------------------------------------------------------------------

/// Generate svg::Params from a slice of pairs.
pub fn element_opts(items: &[(&str, &str)]) -> svg::Params {
    items
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_figure_creation() {}
}
