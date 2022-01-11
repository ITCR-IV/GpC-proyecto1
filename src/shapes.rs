//! Mathematical representations of Points, Lines, and Polygons

use crate::constants::{SCENE_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::window::Window;
use anyhow::{anyhow, Result};

/// Important to note that this is a point in universal, continous coordinates.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point<T> {
    x: T,
    y: T,
}

pub type Universal = f32;
pub type Framebuffer = u32;

fn check_ranges<N: PartialOrd + ToString>(values: Vec<N>, min: N, max: N) -> Result<()> {
    let mut wrong_vals = values.iter().filter(|v| **v < min || **v > max).peekable();
    if wrong_vals.peek().is_some() {
        Err(
            anyhow!("Values for {} type given outside the [{}, {}] range. The following were the erronous ranges:{}", std::any::type_name::<N>(), min.to_string(), max.to_string(), wrong_vals.fold(String::from(""),|acc, v| acc + " " + &v.to_string())),
        )
    } else {
        Ok(())
    }
}

impl Point<Framebuffer> {
    pub fn new(x: Framebuffer, y: Framebuffer) -> Result<Point<Framebuffer>> {
        check_ranges(vec![x], 0, WINDOW_WIDTH - 1)?;
        check_ranges(vec![y], 0, WINDOW_HEIGHT - 1)?;
        Ok(Point { x, y })
    }
}

impl Point<Universal> {
    pub fn new(x: Universal, y: Universal) -> Result<Point<Universal>> {
        check_ranges(vec![x, y], 0.0, SCENE_SIZE as Universal)?;
        Ok(Point { x, y })
    }
}

impl<T: Copy> Point<T> {
    pub fn new_unchecked(x: T, y: T) -> Point<T> {
        Point { x, y }
    }

    pub fn x(&self) -> T {
        self.x
    }
    pub fn y(&self) -> T {
        self.y
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
}

fn is_hex_format(hex: &str) -> bool {
    hex.starts_with('#') && hex.len() == 7 && hex[1..].chars().all(|d| d.is_digit(16))
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Result<Color> {
        check_ranges(vec![r, g, b], 0.0, 1.0)?;
        Ok(Color { r, g, b })
    }

    pub fn from_hex(hex: &str) -> Result<Color> {
        if is_hex_format(hex) {
            Ok(Color::new(
                u8::from_str_radix(&hex[1..=2], 16)? as f32 / 255.0,
                u8::from_str_radix(&hex[3..=4], 16)? as f32 / 255.0,
                u8::from_str_radix(&hex[5..=6], 16)? as f32 / 255.0,
            )?)
        } else {
            Err(anyhow!(
                "from_hex() llamado en string incorrectamente formateado para hexadecimales: '{}'",
                hex
            ))
        }
    }

    pub fn r(&self) -> f32 {
        self.r
    }
    pub fn g(&self) -> f32 {
        self.g
    }
    pub fn b(&self) -> f32 {
        self.b
    }
}

/// Note that a 'Line' isn't a straight 2-point line. It's composed of an arbitrary amount of
/// Points. It can represent the entire border encapsulating a polygon, or a single dot. If a
/// line circles back then the last Point will be equal to the first one.
pub type Line<T> = Vec<Point<T>>;

/// Represents a straight segment, with (x0, y0) being the starting point and (x1, y1) the ending point.
#[derive(Debug)]
pub struct Segment {
    pub x0: u32,
    pub y0: u32,
    pub x1: u32,
    pub y1: u32,
}

//impl Segment {
//    pub fn new(p0: Point, p1: Point) -> Segment {
//        Segment {
//            x0: p0.x(),
//            y0: p0.y(),
//            x1: p1.x(),
//            y1: p1.y(),
//        }
//    }
//}

pub trait LineMethods<T> {
    fn euclidean_length(&self) -> T;
}

pub trait LineClip {
    fn clip_border(
        &self,
        edge: Universal,
        window: &Window,
        intersection: fn(Point<Universal>, Point<Universal>, Universal) -> Point<Universal>,
        inside_edge: fn(&Window, Point<Universal>, Universal) -> bool,
    ) -> Line<Universal>;
}

impl LineMethods<Universal> for Line<Universal> {
    fn euclidean_length(&self) -> Universal {
        self.windows(2)
            .map(|w| ((w[1].x() - w[0].x()).powi(2) + (w[1].y() - w[0].y()).powi(2)).sqrt())
            .sum()
    }
}

impl LineClip for Line<Universal> {
    fn clip_border(
        &self,
        edge: Universal,
        window: &Window,
        intersection: fn(Point<Universal>, Point<Universal>, Universal) -> Point<Universal>,
        inside_edge: fn(&Window, Point<Universal>, Universal) -> bool,
    ) -> Line<Universal> {
        //println!("------------------------\nInput: {:?}", self);
        //println!("Edge: {}", edge);
        let closed = self.first() == self.last();
        let mut clipped = self
            .windows(2)
            .fold(Vec::with_capacity(self.len()), |mut clip, s| {
                match (
                    inside_edge(window, s[0], edge),
                    inside_edge(window, s[1], edge),
                ) {
                    (true, true) => clip.push(s[1]),
                    (true, false) => clip.push(intersection(s[0], s[1], edge)),
                    (false, true) => {
                        clip.extend_from_slice(&[intersection(s[0], s[1], edge), s[1]])
                    }
                    (false, false) => (),
                };
                clip
            });
        if closed {
            if let Some(p) = clipped.last().copied() {
                clipped.insert(0, p)
            }
        }
        //println!("Output: {:?}\n------------------------\n", clipped);
        clipped
    }
}

pub struct Polygon<T> {
    /// The borders being a Vec<Line> doesn't mean that every straight line encapsulating for
    /// example a square is a different border. That would be a polygon considered having just one border. The multiple borders are for polygons that have "holes" in them, like hollowed out circles.
    borders: Vec<Line<T>>,

    /// Border color being "None" just means to not draw an outline when in "color" and "texture"
    /// modes.
    border_color: Option<Color>,

    /// If fill color is "None" it means the polygon shouldn't be filled in and only the lines
    /// should be drawn with Bresenham's.
    fill_color: Option<Color>,

    /// Layer to be drawn on.
    layer: i32,

    /// Id given in the svg.
    id: String,
}

impl<T> Polygon<T> {
    pub fn new(layer: i32, id: String) -> Polygon<T> {
        Polygon {
            borders: Vec::new(),
            border_color: None,
            fill_color: None,
            layer,
            id,
        }
    }

    pub fn new_copy_attributes<U>(&self, borders: Vec<Line<U>>) -> Polygon<U> {
        Polygon {
            id: self.id.clone(),
            borders,
            border_color: self.border_color,
            fill_color: self.fill_color,
            layer: self.layer,
        }
    }

    pub fn add_border(&mut self, border: Line<T>) {
        self.borders.push(border);
    }

    pub fn set_borders(&mut self, borders: Vec<Line<T>>) {
        self.borders = borders;
    }

    pub fn get_borders(&self) -> &Vec<Line<T>> {
        &self.borders
    }

    pub fn get_borders_mut(&mut self) -> &mut Vec<Line<T>> {
        &mut self.borders
    }

    pub fn set_stroke_color(&mut self, color: Option<Color>) {
        self.border_color = color;
    }

    pub fn set_fill_color(&mut self, color: Option<Color>) {
        self.fill_color = color;
    }

    pub fn id(&self) -> &String {
        &self.id
    }
}

impl Polygon<Universal> {
    pub fn scale(mut self, scale: Universal) -> Result<Self> {
        for line in self.borders.iter_mut() {
            for point in line.iter_mut() {
                *point = Point::<Universal>::new(point.x() * scale, point.y() * scale)?
            }
        }
        Ok(self)
    }
}
