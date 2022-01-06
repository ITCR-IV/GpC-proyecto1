mod car;
mod constants;
mod shapes;

use constants::{POINT_SPACING, SVG_SCALE};

fn main() {
    let path = "images/car.svg";
    car::parse_svg(path, SVG_SCALE, POINT_SPACING).unwrap();
}
