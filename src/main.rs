mod car;
mod constants;
mod shapes;
mod window;

use anyhow::{Context, Result};
use car::Car;
use constants::{POINT_SPACING, SCENE_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH, ZOOM_AMOUNT};
use futures::executor::block_on;
use sdl_wrapper::{Event, Keycode};
use window::{Pan, Window};

fn main() -> Result<()> {
    let path = "images/car.svg";
    let car = car::parse_svg(path, SCENE_SIZE, POINT_SPACING)?;
    let window = Window::new("2D World", WINDOW_WIDTH, WINDOW_HEIGHT)?;

    block_on(screen_loop(window, car))?;

    Ok(())
}

async fn screen_loop(mut window: Window, car: Car) -> Result<()> {
    let (mut zoom, mut pan, mut _rotate) = (1.0, Option::<Pan>::None, false);

    'main: loop {
        window.update(&car).await?;
        // Manejo de eventos
        for event in window.get_events() {
            match event {
                // Salirse del programa si se cierra la ventana o estripa Esc
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => break 'main,
                    Keycode::Equals => zoom = 1.0 - ZOOM_AMOUNT,
                    Keycode::Minus => zoom = 1.0 + ZOOM_AMOUNT,
                    Keycode::Up | Keycode::K => pan = Some(Pan::Up),
                    Keycode::Down | Keycode::J => pan = Some(Pan::Down),
                    Keycode::Left | Keycode::H => pan = Some(Pan::Left),
                    Keycode::Right | Keycode::L => pan = Some(Pan::Right),
                    _ => (),
                },
                _ => (),
            }
        }

        if zoom != 1.0 {
            match window.zoom(zoom) {
                Err(err) if zoom > 1.0 => {
                    println!("Can't zoom out anymore: {}", err);
                    Ok(())
                }
                err @ Err(_) if zoom <= 1.0 => err.context("Error zooming into picture"),
                err @ Err(_) => err.context("Extremely weird error in window.zoom() match arms"),
                Ok(_) => Ok(()),
            }?;
            zoom = 1.0;
        }

        if pan.is_some() {
            window
                .pan(pan.unwrap())
                .unwrap_or_else(|err| println!("Pan unsuccesful: {}", err));
            pan = None;
        }
    }
    Ok(())
}
