mod car;
mod constants;
mod shapes;
mod window;

use anyhow::Result;
use car::Car;
use constants::{POINT_SPACING, SCENE_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};
use futures::executor::block_on;
use sdl_wrapper::{Event, Keycode};
use window::Window;

fn main() -> Result<()> {
    let path = "images/car.svg";
    let car = car::parse_svg(path, SCENE_SIZE, POINT_SPACING)?;
    let window = Window::new("2D World", WINDOW_WIDTH, WINDOW_HEIGHT)?;

    block_on(screen_loop(window, car))?;

    Ok(())
}

async fn screen_loop(mut window: Window, car: Car) -> Result<()> {
    window.update(&car).await?;
    'main: loop {
        // Manejo de eventos
        for event in window.get_events() {
            match event {
                // Salirse del programa si se cierra la ventana o estripa Esc
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => (),
            }
        }
    }
    Ok(())
}
