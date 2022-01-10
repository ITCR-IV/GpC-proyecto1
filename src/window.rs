//! Contains the Window class, which represents the window in the computer graphics
//! sense. It wraps the sdl_wrapper ScreenContextManager and implements all the drawing methods.
use anyhow::{anyhow, Result};
use sdl_wrapper::{EventPollIterator, ScreenContextManager};

use crate::constants::SCENE_SIZE;
use crate::shapes::Point;

pub struct Window {
    /// The top-left corner
    min_point: Point,

    /// The bottom-right corner
    max_point: Point,

    screen: ScreenContextManager,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Window> {
        let screen = ScreenContextManager::new(title, width, height)?;

        // center window in the scene
        let window = if height > width {
            let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
            Window {
                min_point: Point::new(spacing, 0.0)?,
                max_point: Point::new(spacing + width as f32, SCENE_SIZE as f32)?,
                screen,
            }
        } else if height < width {
            let spacing = (SCENE_SIZE as f32 - width as f32) / 2.0;
            Window {
                min_point: Point::new(0.0, spacing)?,
                max_point: Point::new(SCENE_SIZE as f32, spacing + height as f32)?,
                screen,
            }
        } else {
            Window {
                min_point: Point::new(0.0, 0.0)?,
                max_point: Point::new(SCENE_SIZE as f32, SCENE_SIZE as f32)?,
                screen,
            }
        };
        Ok(window)
    }

    pub fn present(&mut self) -> Result<()> {
        Ok(self.screen.present()?)
    }

    pub fn get_events(&mut self) -> EventPollIterator {
        self.screen.get_events()
    }
}
