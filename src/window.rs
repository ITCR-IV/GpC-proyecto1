//! Contains the Window class, which represents the window in the computer graphics
//! sense. It wraps the sdl_wrapper ScreenContextManager and implements all the drawing methods.
use anyhow::{anyhow, Context, Result};
use sdl_wrapper::{EventPollIterator, ScreenContextManager};

use crate::car::Car;
use crate::constants::{BACKGROUND_COLOR, PAN_PERCENT, SCENE_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::shapes::{Color, Framebuffer, Line, LineClip, Point, Polygon, Segment, Universal};

use std::cmp::Ordering;

pub enum DisplayMode {
    NoColor,
    ColorFill,
    TextureFill,
    CarTextureFill,
}

pub enum Pan {
    Up,
    Down,
    Left,
    Right,
}

pub struct Window {
    /// The top-left corner
    min_point: Point<Universal>,

    /// The bottom-right corner
    max_point: Point<Universal>,

    screen: ScreenContextManager,

    /// Store background color
    background_color: Color,

    /// Display mode for the car
    display_mode: DisplayMode,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Window> {
        let screen = ScreenContextManager::new(title, WINDOW_WIDTH, WINDOW_HEIGHT)?;
        let background_color = Color::from_hex(BACKGROUND_COLOR)?;
        let display_mode = DisplayMode::TextureFill;

        let window = Window {
            min_point: Point::<Universal>::new(0.0, 0.0)?,
            max_point: Point::<Universal>::new(SCENE_SIZE as f32, SCENE_SIZE as f32)?,
            screen,
            background_color,
            display_mode,
        };

        // center window in the scene
        let window = match height.cmp(&width) {
            Ordering::Greater => {
                let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
                Window {
                    min_point: Point::<Universal>::new(spacing, 0.0)?,
                    max_point: Point::<Universal>::new(spacing + width as f32, SCENE_SIZE as f32)?,
                    ..window
                }
            }
            Ordering::Less => {
                let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
                Window {
                    min_point: Point::<Universal>::new(0.0, spacing)?,
                    max_point: Point::<Universal>::new(SCENE_SIZE as f32, spacing + height as f32)?,
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
        let fb_polys: Vec<Polygon<Framebuffer>> = self.map_to_framebuffer(&self.clip_car(car))?;

        self.no_color_draw(&fb_polys);

        // Finally present changes
        self.screen
            .present()
            .await
            .unwrap_or_else(|err| println!("Error while presenting screen: {}", err));

        Ok(())
    }

    fn no_color_draw(&mut self, fb_polys: &Vec<Polygon<Framebuffer>>) {
        for poly in fb_polys {
            for line in poly.get_borders() {
                for segment in line.windows(2) {
                    let segment = Segment {
                        x0: segment[0].x(),
                        x1: segment[1].x(),
                        y0: segment[0].y(),
                        y1: segment[1].y(),
                    };
                    bresenham_line(&mut self.screen, &segment);
                }
            }
        }
    }

    pub fn zoom(&mut self, zoom: Universal) -> Result<()> {
        let x_c = (self.min_point.x() + self.max_point.x()) / 2.0;
        let y_c = (self.min_point.y() + self.max_point.y()) / 2.0;

        //println!(
        //    "Before zoom '{}' old points are: {:?}\t{:?}",
        //    zoom, self.max_point, self.min_point
        //);
        let min_x = (self.min_point.x() - x_c) * zoom + x_c;
        let min_y = (self.min_point.y() - y_c) * zoom + y_c;
        let max_x = (self.max_point.x() - x_c) * zoom + x_c;
        let max_y = (self.max_point.y() - y_c) * zoom + y_c;

        //println!(
        //    "\nmin_x: {}\tmin_y: {}\nmax_x: {}\tmax_y: {}",
        //    min_x, min_y, max_x, max_y
        //);

        let (min, max): (Point<Universal>, Point<Universal>) = match (
            Point::<Universal>::new(min_x, min_y),
            Point::<Universal>::new(max_x, max_y),
        ) {
            (Err(_), Ok(max)) if zoom > 1.0 => {
                let x_displacement = (min_x - min_x.abs()) / 2.0;
                let y_displacement = (min_y - min_y.abs()) / 2.0;
                println!("x_disp: {}\t y_disp: {}", x_displacement, y_displacement);
                (
                    Point::<Universal>::new(min_x - x_displacement, min_y - y_displacement)
                        .context("The other serious error")?,
                    Point::<Universal>::new(
                        (max_x - x_displacement).min(SCENE_SIZE as Universal),
                        (max_y - y_displacement).min(SCENE_SIZE as Universal),
                    )
                    .context("The other serious error but for max")?,
                )
            }
            (Ok(min), Err(_)) if zoom > 1.0 => {
                let x_displacement = (max_x - SCENE_SIZE as f32).max(0.0);
                let y_displacement = (max_y - SCENE_SIZE as f32).max(0.0);
                println!("x_disp: {}\t y_disp: {}", x_displacement, y_displacement);
                (
                    Point::<Universal>::new(
                        (min_x - x_displacement).max(0.0),
                        (min_y - y_displacement).max(0.0),
                    )
                    .context("Actually a serious error but for min")?,
                    Point::<Universal>::new(max_x - x_displacement, max_y - y_displacement)
                        .context("Actually a serious error")?,
                )
            }
            (Err(_), Err(_)) if zoom > 1.0 => (
                Point::<Universal>::new(0.0, 0.0)?,
                Point::<Universal>::new(SCENE_SIZE as f32, SCENE_SIZE as f32)?,
            ),
            (Err(err), _) | (_, Err(err)) if zoom <= 1.0 => {
                return Err(anyhow!("Error zooming into picture: {}", err));
            }

            (Ok(min), Ok(max)) => (min, max),
            (Err(err), _) | (_, Err(err)) => {
                return Err(anyhow!(
                    "Extremely weird error in window.zoom() match arms: {}",
                    err
                ));
            }
        };

        self.min_point = min;
        self.max_point = max;

        //println!(
        //    "After zoom '{}' new points are: {:?}\t{:?}",
        //    zoom, self.max_point, self.min_point
        //);
        Ok(())
    }

    pub fn pan(&mut self, pan: Pan) -> Result<()> {
        let distance = match pan {
            Pan::Up | Pan::Down => (self.max_point.y() - self.min_point.y()) * PAN_PERCENT,
            Pan::Left | Pan::Right => (self.max_point.x() - self.min_point.x()) * PAN_PERCENT,
        };

        let (max, min) = match pan {
            Pan::Up => (
                Point::<Universal>::new(self.max_point.x(), self.max_point.y() - distance)
                    .context("Límite superior de la escena alcanzado")?,
                Point::<Universal>::new(self.min_point.x(), self.min_point.y() - distance)
                    .context("Límite superior de la escena alcanzado")?,
            ),
            Pan::Down => (
                Point::<Universal>::new(self.max_point.x(), self.max_point.y() + distance)
                    .context("Límite inferior de la escena alcanzado")?,
                Point::<Universal>::new(self.min_point.x(), self.min_point.y() + distance)
                    .context("Límite inferior de la escena alcanzado")?,
            ),
            Pan::Left => (
                Point::<Universal>::new(self.max_point.x() - distance, self.max_point.y())
                    .context("Límite izquierdo de la escena alcanzado")?,
                Point::<Universal>::new(self.min_point.x() - distance, self.min_point.y())
                    .context("Límite izquierdo de la escena alcanzado")?,
            ),
            Pan::Right => (
                Point::<Universal>::new(self.max_point.x() + distance, self.max_point.y())
                    .context("Límite derecho de la escena alcanzado")?,
                Point::<Universal>::new(self.min_point.x() + distance, self.min_point.y())
                    .context("Límite derecho de la escena alcanzado")?,
            ),
        };
        self.min_point = min;
        self.max_point = max;
        Ok(())
    }

    fn map_to_framebuffer(&self, clipped_car: &Car) -> Result<Vec<Polygon<Framebuffer>>> {
        clipped_car
            .iter()
            .map(|polygon| {
                let fb_borders = polygon
                    .get_borders()
                    .iter()
                    .map(|border| -> Result<Line<Framebuffer>> {
                        border
                            .iter()
                            .map(|point| {
                                Point::<Framebuffer>::new(
                                    (WINDOW_WIDTH as Universal * (point.x() - self.min_point.x())
                                        / (self.max_point.x() - self.min_point.x()))
                                    .round() as Framebuffer,
                                    (WINDOW_HEIGHT as Universal * (point.y() - self.min_point.y())
                                        / (self.max_point.y() - self.min_point.y()))
                                    .round() as Framebuffer,
                                )
                                .context(format!(
                                    "Mapping of the point in universal coords '{:?}' to FB",
                                    point
                                ))
                            })
                            .collect::<Result<Line<Framebuffer>>>()
                    })
                    .collect::<Result<Vec<Line<Framebuffer>>>>()
                    .context("Wrong mapping from universal coordinates to framebuffer")?;
                Ok(polygon.new_copy_attributes::<Framebuffer>(fb_borders))
            })
            .collect()
    }

    fn clip_car(&self, car: &Car) -> Car {
        let pre_max_ratio = 1.0 / WINDOW_WIDTH as Universal;
        let pre_max_width = (self.max_point.x() - self.min_point.x()) * pre_max_ratio;
        let pre_max_height = (self.max_point.y() - self.min_point.y()) * pre_max_ratio;
        car.iter()
            .fold(Vec::with_capacity(car.len()), |mut clipped_polys, poly| {
                //println!("id: {}", poly.id());
                let borders = poly.get_borders();
                let borders = borders.iter().fold(
                    Vec::with_capacity(borders.len()),
                    |mut clipped_borders: Vec<Line<Universal>>,
                     border: &Line<Universal>|
                     -> Vec<Line<Universal>> {
                        let clipped_border = border
                            .clip_border(
                                self.max_point.x() - pre_max_width,
                                self,
                                intersection_vertical,
                                Self::inside_max_x_edge,
                            )
                            .clip_border(
                                self.max_point.y() - pre_max_height,
                                self,
                                intersection_horizontal,
                                Self::inside_max_y_edge,
                            )
                            .clip_border(
                                self.min_point.x(),
                                self,
                                intersection_vertical,
                                Self::inside_min_x_edge,
                            )
                            .clip_border(
                                self.min_point.y(),
                                self,
                                intersection_horizontal,
                                Self::inside_min_y_edge,
                            );
                        // This removes the borders that are fully out of frame
                        if clipped_border.len() > 0 {
                            clipped_borders.push(clipped_border);
                        }
                        clipped_borders
                    },
                );
                // This will remove the polygons whose borders are all fully out of frame
                if borders.len() > 0 {
                    clipped_polys.push(poly.new_copy_attributes(borders));
                }
                clipped_polys
            })
    }

    fn contains(&self, point: Point<Universal>) -> bool {
        point.x() >= self.min_point.x()
            && point.x() < self.max_point.x()
            && point.y() >= self.min_point.y()
            && point.y() < self.max_point.y()
    }

    fn inside_min_x_edge(&self, point: Point<Universal>, edge: Universal) -> bool {
        point.x() >= edge
    }
    fn inside_min_y_edge(&self, point: Point<Universal>, edge: Universal) -> bool {
        point.y() >= edge
    }
    fn inside_max_x_edge(&self, point: Point<Universal>, edge: Universal) -> bool {
        point.x() < edge
    }
    fn inside_max_y_edge(&self, point: Point<Universal>, edge: Universal) -> bool {
        point.y() < edge
    }
}

fn intersection_horizontal(
    p0: Point<Universal>,
    p1: Point<Universal>,
    y_edge: f32,
) -> Point<Universal> {
    let x = if p0.x() == p1.x() {
        p0.x()
    } else {
        let m = (p1.y() - p0.y()) / (p1.x() - p0.x());
        let b = p0.y() - m * p0.x();
        (y_edge - b) / m
        //let x = (y_edge - b) / m;
        //if x > 1000.0 {
        //    println!(
        //        "-------\ny = mx + b\ny:{}\tm:{}\tx:{}\tb:{}\np0:{:?}\tp1:{:?}\n-------",
        //        y_edge, m, x, b, p0, p1
        //    );
        //}
        //if x.is_nan() {
        //    println!(
        //        "-------\ny = mx + b\ny:{}\tm:{}\tx:{}\tb:{}\np0:{:?}\tp1:{:?}\n-------",
        //        y_edge, m, x, b, p0, p1
        //    );
        //}
        //x
    };
    Point::<Universal>::new_unchecked(x, y_edge)
}

fn intersection_vertical(
    p0: Point<Universal>,
    p1: Point<Universal>,
    x_edge: f32,
) -> Point<Universal> {
    let m = (p1.y() - p0.y()) / (p1.x() - p0.x());
    let b = p0.y() - m * p0.x();
    Point::<Universal>::new_unchecked(x_edge, m * x_edge + b)
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
