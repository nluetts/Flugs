#![allow(unused)]

use std::{collections::HashMap, fmt::Write};

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
            let ax_tags = ax.to_tags(self);
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
    draw_legend: bool,
    draw_xaxis: bool,
    draw_yaxis: bool,
    elements: Vec<Box<dyn Element>>,
    xlabel: String,
    ylabel: String,
    height: f64,
    limits: [f64; 4],
    plots: Vec<LinePlot>,
    style: svg::Params,
    pub ticks: Ticks,
    /// u coordinate for placement in Figure, normalized to [0, 1]
    u: f64,
    /// v coordinate for placement in Figure, normalized to [0, 1]
    v: f64,
    width: f64,
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
            draw_legend: false,
            draw_xaxis: true,
            draw_yaxis: true,
            plots: Vec::new(),
            elements: Vec::new(),
            style: element_opts(&[("fill", "none"), ("stroke", "none")]),
            xlabel: "".to_string(),
            ylabel: "".to_string(),
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
        self.plots.push(line);
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
        self.xlabel = text.to_owned();
        self
    }

    pub fn with_ylim(mut self, ymin: f64, ymax: f64) -> Self {
        self.ylim(ymin, ymax);
        self.autoticks();
        self
    }

    pub fn with_ylabel(mut self, text: &str) -> Self {
        self.ylabel = text.to_owned();
        self
    }

    pub fn draw_xaxis(mut self, should_draw: bool) -> Self {
        self.draw_xaxis = should_draw;
        self
    }

    pub fn draw_yaxis(mut self, should_draw: bool) -> Self {
        self.draw_yaxis = should_draw;
        self
    }

    pub fn with_x_minor_ticks(mut self, num: usize) -> Self {
        self.ticks.x_num_minor = num;
        self
    }

    pub fn with_y_minor_ticks(mut self, num: usize) -> Self {
        self.ticks.y_num_minor = num;
        self
    }

    pub fn with_legend(mut self, flag: bool) -> Self {
        self.draw_legend = flag;
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

        // Make room for the ytick labels by resizing and moving the axis.
        // TODO: This will make the axis shrink with each call to this function.
        // let shift = {
        //     let width = self.ticks.y_tick_label_character_width();
        //     width as f64 * 1e-3
        // };
        // self.u += shift;
        // self.width -= shift;
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
        children.extend(
            self.plots
                .iter()
                .map(|el| el.to_tags(self, fig))
                .reduce(|mut ts, next_ts| {
                    ts.extend(next_ts);
                    ts
                })
                .unwrap_or(Vec::new()),
        );

        // Add axis labels.
        if self.draw_xaxis && !self.xlabel.is_empty() {
            let xlabel = Text {
                text: self.xlabel.to_owned(),
                u: 0.5,
                v: -0.1,
                angle: 0.0,
                style: element_opts(&[("text-anchor", "middle")]),
            }
            .to_tags(&self, &fig);
            children.extend(xlabel);
        }
        if self.draw_yaxis && !self.ylabel.is_empty() {
            let y_tick_label_width = self.ticks.y_tick_label_character_width();
            let ylabel = Text {
                text: self.ylabel.to_owned(),
                u: -0.075 - 0.005 * y_tick_label_width as f64,
                v: 0.5,
                angle: 270.0,
                style: element_opts(&[("text-anchor", "middle")]),
            }
            .to_tags(&self, &fig);
            children.extend(ylabel);
        }

        if self.draw_legend {
            let mut legend_elements = Vec::new();
            for (i, p) in self.plots.iter().filter(|p| !p.name.is_empty()).enumerate() {
                let label = Text {
                    text: p.name.to_owned(),
                    u: 0.95,
                    v: 0.95 - (i as f64 * 0.05),
                    angle: 0.0,
                    style: element_opts(&[
                        (
                            "fill",
                            p.style.get("stroke").unwrap_or(&"black".to_string()),
                        ),
                        ("text-anchor", "end"),
                        ("font-size", "10pt"),
                    ]),
                };
                legend_elements.push(label);
            }
            children.extend(
                legend_elements
                    .iter()
                    .flat_map(|lab| lab.to_tags(self, fig)),
            );
        }
        children.extend(self.ticks.to_tags(self, fig));
        let mut ax_rect = Tag::<svg::Rect>::new(
            w * self.u,
            h * self.v,
            self.width * w,
            self.height * h,
            Some(self.style.clone()),
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
    ///
    /// Additionally, the function splits the data into segments whenever
    /// NaN values are encountered, because NaNs in the SVG output lead
    /// to the polyline not being drawn fully.
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
        for i in 0..(n - 1) {
            // If a datapoint is infity or NaN, we start a new segment
            // and ignore the point.
            if !xs[i].is_finite() || !ys[i].is_finite() {
                if !segments.last().unwrap().0.is_empty() {
                    segments.push((Vec::<f64>::new(), Vec::<f64>::new()));
                }
                continue;
            }
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
        Axis::new(0.125, 0.1, 0.875, 0.8)
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
    style: svg::Params,
}

impl Element for Text {
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (x, y, _, _) = ax.transformations(fig);

        let (x, y) = (x(self.u), y(1.0 - self.v));
        let mut style = Vec::new();
        style.extend(self.style.iter().map(|(k, v)| (&k[..], &v[..])));

        vec![Box::new(Tag::<svg::Text>::new(
            x,
            y,
            self.angle,
            &self.text,
            svg::opts(&style),
        ))]
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.style.insert(key.to_string(), value.to_string());
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
    pub xpos: Vec<f64>,
    pub ypos: Vec<f64>,
    pub x_num_minor: usize,
    pub y_num_minor: usize,
    color: String,
    linewidth: f64,
    style: svg::Params,
}

impl Default for Ticks {
    fn default() -> Self {
        Self {
            xpos: Vec::new(),
            ypos: Vec::new(),
            x_num_minor: 3,
            y_num_minor: 3,
            color: "black".to_string(),
            linewidth: 1.0,
            style: Params::new(),
        }
    }
}

impl Element for Ticks {
    fn to_tags(&self, ax: &Axis, fig: &Figure) -> Vec<Box<dyn svg::RenderTag>> {
        let (x, y, u, v) = ax.transformations(fig);
        let [xmin, xmax, ymin, ymax] = ax.limits_ordered();

        let width_param = format!("{}", self.linewidth);
        let style = [
            ("stroke", &self.color[..]),
            ("stroke-width", &width_param[..]),
        ];
        let style_minor = [("stroke", "lightgray"), ("stroke-width", &width_param[..])];
        let style_xtick_label = [("text-anchor", "middle")];
        let style_ytick_label = [("text-anchor", "end")];

        let mut xticks: Vec<Box<dyn svg::RenderTag>> = Vec::new();
        let mut yticks: Vec<Box<dyn svg::RenderTag>> = Vec::new();

        let (xtick_labels, ytick_labels) = self.format_ticks();

        if ax.draw_xaxis {
            let mut iter = self.xpos.iter().zip(xtick_labels).peekable();
            while let Some((&xi, li)) = iter.next() {
                let lt = Tag::<svg::Line>::new(x(u(xi)), x(u(xi)), y(0.99), y(1.01), opts(&style));
                let tt = Tag::<svg::Text>::new(
                    x(u(xi)),
                    y(1.05),
                    0.0,
                    &li,
                    opts(&style_xtick_label.clone()),
                );
                xticks.push(Box::new(lt));
                xticks.push(Box::new(tt));

                // Draw minor ticks.
                if let Some((&xj, _)) = iter.peek() {
                    if self.x_num_minor == 0 {
                        continue;
                    }
                    let step = (xj - xi) / (self.x_num_minor as f64 + 1.0);
                    for i in 1..=self.x_num_minor {
                        let lt = Tag::<svg::Line>::new(
                            x(u(xi + i as f64 * step)),
                            x(u(xi + i as f64 * step)),
                            y(0.99),
                            y(1.01),
                            opts(&style_minor),
                        );
                        xticks.push(Box::new(lt));
                    }
                }
            }
        }

        if ax.draw_yaxis {
            let mut iter = self.ypos.iter().zip(ytick_labels).peekable();
            while let Some((&yi, li)) = iter.next() {
                let lt =
                    Tag::<svg::Line>::new(x(-0.005), x(0.005), y(v(yi)), y(v(yi)), opts(&style));
                let tt = Tag::<svg::Text>::new(
                    x(-0.03),
                    y(v(yi) + 0.02),
                    0.0,
                    &li,
                    opts(&style_ytick_label.clone()),
                );
                yticks.push(Box::new(lt));
                yticks.push(Box::new(tt));

                // Draw minor ticks.
                if let Some((&yj, _)) = iter.peek() {
                    if self.y_num_minor == 0 {
                        continue;
                    }
                    let step = (yj - yi) / (self.y_num_minor as f64 + 1.0);
                    for i in 1..=self.y_num_minor {
                        let lt = Tag::<svg::Line>::new(
                            x(-0.005),
                            x(0.005),
                            y(v(yi + i as f64 * step)),
                            y(v(yi + i as f64 * step)),
                            opts(&style_minor),
                        );
                        yticks.push(Box::new(lt));
                    }
                }
            }
        }

        xticks.extend(yticks);
        xticks
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.style.insert(key.to_string(), value.to_string());
    }

    fn identifier(&self) -> &str {
        "Ticks"
    }
}

impl Ticks {
    fn format_ticks(&self) -> (Vec<String>, Vec<String>) {
        let xtick_labels = format_ticks(&self.xpos);
        let ytick_labels = format_ticks(&self.ypos);

        (xtick_labels, ytick_labels)
    }

    fn y_tick_label_character_width(&self) -> usize {
        let (_, labels) = self.format_ticks();
        labels.iter().map(|lab| lab.len()).max().unwrap_or(0)
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
    style: svg::Params,
    name: String,
}

impl LinePlot {
    pub fn new(xs: &[f64], ys: &[f64]) -> Self {
        Self {
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            style: svg::Params::new(),
            name: String::new(),
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.style.insert("stroke".to_string(), color.to_string());
        self
    }

    pub fn with_linewidth(mut self, linewidth: f64) -> Self {
        self.style
            .insert("stroke-width".to_string(), format!("{linewidth}"));
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name.drain(..);
        self.name.write_str(name);
        self
    }

    pub fn set_color(&mut self, color: &str) {
        self.style.insert("stroke".to_string(), color.to_string());
    }

    pub fn set_linewidth(&mut self, linewidth: f64) {
        self.style
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
        let mut svg_tags: Vec<Box<dyn svg::RenderTag>> = Vec::new();
        for (xs, ys) in ax.segment_lineplot_data(self).into_iter() {
            let xs = xs.iter().map(|xi| x(u(*xi)));
            let ys = ys.iter().map(|yi| y(v(*yi)));
            svg_tags.push(Box::new(Tag::<svg::Polyline>::new(
                xs,
                ys,
                Some(self.style.clone()),
            )));
        }

        svg_tags
    }

    fn add_svg_property(&mut self, key: &str, value: &str) {
        self.style.insert(key.to_string(), value.to_string());
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

fn format_ticks(tick_positions: &[f64]) -> Vec<String> {
    let (magnitude_min, magnitude_max, mag_increment) =
        ticks_magnitude_and_increment(tick_positions);

    let fmt_fun = match (magnitude_min, magnitude_max, mag_increment) {
        // The min/max magnitude are not used right now, but maybe I realise
        // I need them in the future. Formatting ticks is tricky ...
        (_, _, i8::MIN..=-4) => |z| format!("{z:.2e}"),
        (_, _, -3) => |z| format!("{z:.4}"),
        (_, _, -2) => |z| format!("{z:.3}"),
        (_, _, -1) => |z| format!("{z:.1}"),
        (_, _, 0) => |z| format!("{z:.1}"),
        (_, _, 1..=4) => |z| format!("{z:.0}"),
        (_, _, 5..=i8::MAX) => |z| format!("{z:.0e}"),
    };

    tick_positions.iter().map(fmt_fun).collect()
}

fn ticks_magnitude_and_increment(tick_positions: &[f64]) -> (i8, i8, i8) {
    let (magnitude_min, magnitude_max) =
        tick_positions
            .iter()
            .filter(|y| **y != 0.0)
            .fold((f64::MAX, f64::MIN), |mut acc, x| {
                let mag_x = x.abs().log10();
                if mag_x < acc.0 {
                    acc.0 = mag_x
                };
                if mag_x > acc.1 {
                    acc.1 = mag_x
                };
                acc
            });
    let mag_increment = {
        let n = tick_positions.len() - 1;
        let mut acc = 0.0;
        for (zi, zj) in tick_positions.iter().zip(tick_positions.iter().skip(1)) {
            acc += (zj - zi).abs();
        }
        (acc / n as f64).log10()
    };
    (
        magnitude_min.floor() as i8,
        magnitude_max.floor() as i8,
        mag_increment.floor() as i8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticks_magnitude_and_increment_1() {
        let pos = vec![0.001, 0.002, 0.003, 0.004];
        let (magnitude_min, magnitude_max, mag_increment) = ticks_magnitude_and_increment(&pos);
        assert_eq!(mag_increment, -3);
        assert_eq!(magnitude_min, -3);
        assert_eq!(magnitude_max, -3);
    }

    #[test]
    fn test_ticks_magnitude_and_increment_2() {
        let pos = vec![0.001, 1.002, 2.003, 3.004];
        let (magnitude_min, magnitude_max, mag_increment) = ticks_magnitude_and_increment(&pos);
        assert_eq!(mag_increment, 0);
        assert_eq!(magnitude_min, -3);
        assert_eq!(magnitude_max, 0);
    }

    #[test]
    fn test_ticks_magnitude_and_increment_3() {
        let pos = vec![10000.0, 12000.0, 14000.0, 16000.0, 18000.0, 20000.0];
        let (magnitude_min, magnitude_max, mag_increment) = ticks_magnitude_and_increment(&pos);
        assert_eq!(mag_increment, 3);
        assert_eq!(magnitude_min, 4);
        assert_eq!(magnitude_max, 4);
    }
}
