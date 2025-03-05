#![allow(unused)]
// This code is a thin Rust wrapper to handle SVG tags
// and render the results to *.svg files.

use std::{
    fmt::{Display, Write},
    io::Seek,
};

// ----------------------------------------------------------------------------
//
//
// Rust representation and rendering of SVG tags.
//
//
// ----------------------------------------------------------------------------

pub type Params = std::collections::HashMap<String, String>;

pub trait RenderTag: std::fmt::Debug {
    fn render(&self, buf: &mut String);
}

impl<T> RenderTag for Tag<T>
where
    T: Identifier + std::fmt::Debug,
{
    fn render(&self, buf: &mut String) {
        write!(buf, "<{}", self.kind.identifier()).expect(FAILED_STRING_WRITE);
        for (k, v) in self.parameters.iter() {
            write!(buf, " {k}=\"{v}\"").expect(FAILED_STRING_WRITE);
        }
        if !self.style.is_empty() {
            write!(buf, " style=\"").expect(FAILED_STRING_WRITE);
            for (k, v) in self.style.iter() {
                write!(buf, "{k}:{v};").expect(FAILED_STRING_WRITE);
            }
            // Remove last surplus space.
            buf.pop();
            write!(buf, "\"").expect(FAILED_STRING_WRITE);
        }
        if !self.closing {
            write!(buf, " /").expect(FAILED_STRING_WRITE);
        }
        write!(buf, ">").expect(FAILED_STRING_WRITE);
        for c in self.children.iter() {
            c.render(buf);
        }
        if self.closing {
            write!(buf, "</{}>", self.kind.identifier()).expect(FAILED_STRING_WRITE);
        }
    }
}

impl RenderTag for String {
    fn render(&self, buf: &mut String) {
        buf.push_str(&self);
    }
}

pub fn render(svg_tag: &Tag<SVG>) -> String {
    let mut raw_svg = String::new();
    svg_tag.render(&mut raw_svg);
    raw_svg
}

#[derive(Debug)]
pub struct Tag<T>
where
    T: std::fmt::Debug,
{
    parameters: Params,
    style: Params,
    children: Vec<Box<dyn RenderTag>>,
    closing: bool,
    kind: T,
}

impl<T> Tag<T>
where
    T: std::fmt::Debug,
{
    pub fn add_child(&mut self, child: impl RenderTag + 'static) {
        self.children.push(Box::new(child));
    }
    pub fn add_children(&mut self, children: Vec<Box<dyn RenderTag>>) {
        self.children.extend(children);
    }
}

// ----------------------------------------------------------------------------
//
//
// Creation of `new` tags.
//
//
// ----------------------------------------------------------------------------

impl Tag<SVG> {
    pub fn new(width: u64, height: u64, style: Option<Params>) -> Self {
        let children = Vec::new();
        let mut parameters: Params = [
            ("width", format!("{width}")),
            ("height", format!("{height}")),
            ("viewBox", format!("0 0 {width} {height}")),
            ("xmlns", "http://www.w3.org/2000/svg".to_string()),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: true,
            kind: SVG {},
        }
    }
}

impl Tag<Circle> {
    pub fn new(cx: f64, cy: f64, r: f64, style: Option<Params>) -> Self {
        let children = Vec::new();
        let mut parameters: Params = [
            ("cx", format!("{cx}")),
            ("cy", format!("{cy}")),
            ("r", format!("{r}")),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: false,
            kind: Circle {},
        }
    }
}

impl Tag<Rect> {
    pub fn new(x: f64, y: f64, width: f64, height: f64, style: Option<Params>) -> Self {
        let children = Vec::new();
        let mut parameters: Params = [
            ("x", format!("{x}")),
            ("y", format!("{y}")),
            ("width", format!("{width}")),
            ("height", format!("{height}")),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: false,
            kind: Rect {},
        }
    }
}

impl Tag<Text> {
    pub fn new(x: f64, y: f64, angle: f64, text: &str, style: Option<Params>) -> Self {
        let children = Vec::new();
        let mut parameters: Params = [("transform", format!("translate({x},{y}) rotate({angle})"))]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let mut res = Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: true,
            kind: Text {},
        };

        res.add_child(text.to_string());
        res
    }
}

impl Tag<Line> {
    pub fn new(x1: f64, x2: f64, y1: f64, y2: f64, style: Option<Params>) -> Self {
        let children = Vec::new();
        let mut parameters: Params = [
            ("x1", format!("{x1}")),
            ("x2", format!("{x2}")),
            ("y1", format!("{y1}")),
            ("y2", format!("{y2}")),
            ("stroke", format!("black")),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

        Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: false,
            kind: Line {},
        }
    }
}
impl Tag<Polyline> {
    pub fn new(
        xs: impl IntoIterator<Item = f64>,
        ys: impl IntoIterator<Item = f64>,
        style: Option<Params>,
    ) -> Self {
        let children = Vec::new();
        let mut raw_points = String::new();
        for (x, y) in xs.into_iter().zip(ys.into_iter()) {
            write!(raw_points, "{x},{y} ");
        }
        let mut parameters: Params = [("points", raw_points), ("fill", "none".to_string())]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        Self {
            parameters,
            style: style.unwrap_or(Params::new()),
            children,
            closing: false,
            kind: Polyline {},
        }
    }
}

// ----------------------------------------------------------------------------
//
//
// SVG tag kinds and their str representation (`identifier`)
//
//
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SVG {}
#[derive(Debug)]
pub struct Circle {}
#[derive(Debug)]
pub struct Rect {}
#[derive(Debug)]
pub struct Text {}
#[derive(Debug)]
pub struct Line {}
#[derive(Debug)]
pub struct Polyline {}

trait Identifier {
    fn identifier(&self) -> &'static str;
}

impl Identifier for SVG {
    fn identifier(&self) -> &'static str {
        "svg"
    }
}

impl Identifier for Circle {
    fn identifier(&self) -> &'static str {
        "circle"
    }
}

impl Identifier for Rect {
    fn identifier(&self) -> &'static str {
        "rect"
    }
}

impl Identifier for Text {
    fn identifier(&self) -> &'static str {
        "text"
    }
}

impl Identifier for Line {
    fn identifier(&self) -> &'static str {
        "line"
    }
}

impl Identifier for Polyline {
    fn identifier(&self) -> &'static str {
        "polyline"
    }
}

const FAILED_STRING_WRITE: &str = "Failed to write into string.";

pub fn opts(items: &[(&str, &str)]) -> Option<Params> {
    Some(
        items
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn it_works() {
        let mut svg = Tag::<SVG>::new(400, 300, None);
        let rect = Tag::<Rect>::new(0.0, 0.0, 400.0, 300.0, opts(&[("fill", "red")]));
        let text = Tag::<Text>::new(200.0, 150.0, 0.0, "This is a Test.", None);
        svg.add_child(rect);
        svg.add_child(text);

        println!("{}", render(&svg));
    }
}
