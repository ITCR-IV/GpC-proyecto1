mod car;
mod constants;
mod shapes;

use constants::{POINT_SPACING, SCENE_SIZE};

fn main() {
    let path = "images/car.svg";
    car::parse_svg(path, SCENE_SIZE, POINT_SPACING).unwrap();
}
