//! Contains the Window class, which represents the window in the computer graphics
//! sense. It wraps the sdl_wrapper ScreenContextManager and implements all the drawing methods.
use anyhow::{anyhow, Result};
use sdl_wrapper::{EventPollIterator, ScreenContextManager};

use crate::car::Car;
use crate::constants::{BACKGROUND_COLOR, SCENE_SIZE};
use crate::shapes::{Color, Line, LineMethods, Point, Polygon, Segment};

use std::cmp::Ordering;

pub enum DisplayMode {
    NoColor,
    ColorFill,
    TextureFill,
    CarTextureFill,
}

pub struct Window {
    /// The top-left corner
    min_point: Point,

    /// The bottom-right corner
    max_point: Point,

    screen: ScreenContextManager,

    /// Store background color
    background_color: Color,

    /// Display mode for the car
    display_mode: DisplayMode,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Window> {
        let screen = ScreenContextManager::new(title, width, height)?;
        let background_color = Color::from_hex(BACKGROUND_COLOR)?;
        let display_mode = DisplayMode::TextureFill;

        let window = Window {
            min_point: Point::new(0.0, 0.0)?,
            max_point: Point::new(SCENE_SIZE as f32, SCENE_SIZE as f32)?,
            screen,
            background_color,
            display_mode,
        };

        // center window in the scene
        let window = match height.cmp(&width) {
            Ordering::Greater => {
                let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
                Window {
                    min_point: Point::new(spacing, 0.0)?,
                    max_point: Point::new(spacing + width as f32, SCENE_SIZE as f32)?,
                    ..window
                }
            }
            Ordering::Less => {
                let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
                Window {
                    min_point: Point::new(0.0, spacing)?,
                    max_point: Point::new(SCENE_SIZE as f32, spacing + height as f32)?,
                    ..window
                }
            }
            Ordering::Equal => window,
        };

        Ok(window)
    }

    pub fn get_events(&mut self) -> EventPollIterator {
        self.screen.get_events()
    }

    pub async fn update(&mut self, car: &Car) -> Result<()> {
        // First clear background
        match self.display_mode {
            DisplayMode::NoColor => {
                self.screen.clear(0.9);
            }
            _ => {
                self.screen.clear_with_rgb(
                    self.background_color.r(),
                    self.background_color.g(),
                    self.background_color.b(),
                );
            }
        }

        // Then paint car
        // TODO: remove empty shit (polygons may well be fully outside of window)
        let cut_polys: Vec<Polygon> = self.clip_car(car);
        for poly in cut_polys {
            for line in poly.get_borders() {
                for segment in line.windows(2) {
                    let segment = Segment {
                        x0: segment[0].x().round() as u32,
                        x1: segment[1].x().round() as u32,
                        y0: segment[0].y().round() as u32,
                        y1: segment[1].y().round() as u32,
                    };
                    bresenham_line(&mut self.screen, &segment);
                }
            }
        }

        // Finally present changes
        self.screen
            .present()
            .await
            .unwrap_or_else(|err| println!("Error while presenting screen: {}", err));

        Ok(())
    }

    fn clip_car(&self, car: &Car) -> Vec<Polygon> {
        car.iter()
            .fold(Vec::with_capacity(car.len()), |mut clipped_polys, poly| {
                let borders = poly.get_borders();
                let borders = borders.iter().fold(
                    Vec::with_capacity(borders.len()),
                    |mut clipped_borders: Vec<Line>, border: &Line| -> Vec<Line> {
                        let clipped_border = border
                            .clip_border(
                                self.max_point.x(),
                                self,
                                intersection_vertical,
                                inside_max_edge,
                            )
                            .clip_border(
                                self.max_point.y(),
                                self,
                                intersection_horizontal,
                                inside_max_edge,
                            )
                            .clip_border(
                                self.min_point.x(),
                                self,
                                intersection_vertical,
                                inside_min_edge,
                            )
                            .clip_border(
                                self.min_point.y(),
                                self,
                                intersection_horizontal,
                                inside_min_edge,
                            );
                        clipped_borders.push(clipped_border);
                        clipped_borders
                    },
                );
                clipped_polys.push(poly.new_copy_attributes(borders));
                clipped_polys
            })
    }

    fn contains(&self, point: Point) -> bool {
        point.x() >= self.min_point.x()
            && point.x() <= self.max_point.x()
            && point.y() >= self.min_point.y()
            && point.y() <= self.max_point.y()
    }
}

fn inside_min_edge(window: &Window, point: Point, edge: f32) -> bool {
    match edge {
            edge if edge == window.min_point.x() => point.x() >= edge,
            edge if edge == window.min_point.y() => point.y() >= edge,
            weird_edge => panic!("The edge given to inside_min_edge() doesn't match any of the current window min edges (edge = '{}')", weird_edge)
        }
}
fn inside_max_edge(window: &Window, point: Point, edge: f32) -> bool {
    match edge {
            edge if edge == window.max_point.x() => point.x() <= edge,
            edge if edge == window.max_point.y() => point.y() <= edge,
            weird_edge => panic!("The edge given to inside_max_edge() doesn't match any of the current window max edges (edge = '{}')", weird_edge)
        }
}
fn intersection_horizontal(p0: Point, p1: Point, y_edge: f32) -> Point {
    let m = (p1.y() - p0.y()) / (p1.x() - p0.x());
    let b = p0.y() - m * p0.x();
    Point::new_unchecked((y_edge - b) / m, y_edge)
}

fn intersection_vertical(p0: Point, p1: Point, x_edge: f32) -> Point {
    let m = (p1.y() - p0.y()) / (p1.x() - p0.x());
    let b = p0.y() - m * p0.x();
    Point::new_unchecked(x_edge, m * x_edge + b)
}

/// Implementation of the bresenham method to draw lines
fn bresenham_line(screen: &mut ScreenContextManager, segment: &Segment) {
    // Check for which type of octant we're on
    if (segment.y1 as i32 - segment.y0 as i32).abs() < (segment.x1 as i32 - segment.x0 as i32).abs()
    {
        // Octants 1, 4, 5, 8
        if segment.x1 > segment.x0 {
            // Octants 1, 8
            bresenham_horizontal(screen, segment.x0, segment.y0, segment.x1, segment.y1);
        } else {
            // Octants 4, 5
            // We must switch the order of the variables with which we call the helper function
            bresenham_horizontal(screen, segment.x1, segment.y1, segment.x0, segment.y0);
        }
    } else {
        // Octants 2, 3, 6, 7
        if segment.y1 > segment.y0 {
            // Octants 2, 3
            bresenham_vertical(screen, segment.x0, segment.y0, segment.x1, segment.y1);
        } else {
            // Octants 6, 7
            bresenham_vertical(screen, segment.x1, segment.y1, segment.x0, segment.y0);
        }
    }
}
fn bresenham_horizontal(screen: &mut ScreenContextManager, x0: u32, y0: u32, x1: u32, y1: u32) {
    let dy = y1 as i32 - y0 as i32;
    // Check for decreasing horizontal quadrants (5, 8)
    let (yi, dy) = if dy < 0 { (-1, -dy) } else { (1, dy) };

    let dx = (x1 - x0) as i32;

    let delta_h = 2 * dy; // delta_h = horizontal movement
    let delta_d = 2 * (dy - dx); // delta_d = diagonal movement

    let mut y = y0 as i32;
    let mut d = 2 * dy - dx;

    for x in x0..x1 {
        screen.plot_pixel(x, y as u32);
        if d > 0 {
            y += yi;
            d += delta_d;
        } else {
            d += delta_h;
        }
    }
}

fn bresenham_vertical(screen: &mut ScreenContextManager, x0: u32, y0: u32, x1: u32, y1: u32) {
    let dx = x1 as i32 - x0 as i32;
    // Check for backwards vertical quadrants (3, 6)
    let (xi, dx) = if dx < 0 { (-1, -dx) } else { (1, dx) };

    let dy = (y1 - y0) as i32;

    let delta_v = 2 * dx; // Vertical movement
    let delta_d = 2 * (dx - dy); // Diagonal movement

    let mut x = x0 as i32;
    let mut d = 2 * dx - dy;

    for y in y0..y1 {
        screen.plot_pixel(x as u32, y);
        if d > 0 {
            x += xi;
            d += delta_d;
        } else {
            d += delta_v;
        }
    }
}
