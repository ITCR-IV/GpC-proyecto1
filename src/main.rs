mod car;
mod constants;
mod shapes;

use anyhow::Result;
use constants::{POINT_SPACING, SCENE_SIZE};

fn main() -> Result<()> {
    let path = "images/car.svg";
    car::parse_svg(path, SCENE_SIZE, POINT_SPACING)?;

    Ok(())
}
