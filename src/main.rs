mod car;
mod constants;
mod shapes;

use constants::*;

fn main() {
    let path = "images/car.svg";
    car::parse_svg(path, SVG_SCALE, POINT_SPACING).unwrap();
}
