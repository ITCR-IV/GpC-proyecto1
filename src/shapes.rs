/// Important to note that this is a poin in universal, continous coordinates.
pub struct Point {
    x: f32,
    y: f32,
}

/// Note that a 'Line' isn't a straight 2-point line. It's composed of an arbitrary amount of
/// Points. It can represent the entire border encapsulating a polygon. If a line circles back then
/// the last Point will be equal to the first one.
pub type Line = Vec<Point>;

pub struct Polygon {
    /// The borders being a Vec<Line> doesn't mean that every straight line encapsulating for
    /// example a square is a different border. That would be a polygon considered having just one border. The multiple borders are for polygons that have "holes" in them, like hollowed out circles.
    borders: Vec<Line>,

    /// Border color being "None" just means to not draw an outline when in "color" and "texture"
    /// modes.
    border_color: Option<f32>,

    /// If fill color is "None" it means the polygon shouldn't be filled in and only the lines
    /// should be drawn with Bresenham's.
    fill_color: Option<f32>,

    /// Layer to be drawn on.
    layer: i32,

    /// Id given in the svg.
    id: String,
}
