#![allow(non_upper_case_globals)]

use anyhow::{anyhow, Context, Result};
use svg::node::element::path::{Command, Data};
use svg::node::element::tag::{Circle, Ellipse, Group, Path, Type, SVG};
use svg::parser::{Event, Parser};

use crate::shapes::Polygon;

pub struct Car {
    polygons: Vec<Polygon>,
    height: u32,
    width: u32,
}

/// Function made to specifically parse the "car.svg" file and return a "Car" object.
pub fn parse_svg(path: &str, scaling: f32, distance: f32) -> Result<Car> {
    let mut content = String::new();
    let (parser, car) = init_svg(path, scaling, &mut content)?;

    let mut layer: i32 = 0;

    for event in parser {
        match event {
            // Group = layers
            Event::Tag(Group, Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("group no trae id (layer sin número)"))?;
                println!("Layer: '{}'", id);
                layer = id
                    .parse()
                    .context("id de 'group' (layer) no se pudo parsear a i32")?;
            }

            // Path = líneas/curvas
            Event::Tag(Path, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("path no trae id"))?;
                println!("Path id: {}", id);
                //let data = attributes.get("d").unwrap();
                //let data = Data::parse(data).unwrap();
                //for command in data.iter() {
                //    match command {
                //        &Command::Move(..) => println!("Move!"),
                //        &Command::Line(..) => println!("Line!"),
                //        _ => {}
                //    }
                //}
            }
            Event::Tag(Circle, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("circle no trae id"))?;
                println!("Circle id: {}", id);
            }
            Event::Tag(Ellipse, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("ellipse no trae id"))?;
                println!("Ellipse id: {}", id);
            }
            // unhandled
            Event::Tag(tag, Type::Start | Type::Empty, _) => {
                println!("!!!Tag sin manejar: {}", tag);
            }
            other => {
                println!("unimportant event: {:?}", other);
            }
        }
    }

    Ok(car)
}

fn approximate_path() {}
fn approximate_circle() {}
fn approximate_ellipse() {}

/// This function parse the initial lines of the "car.svg" file, ignoring anything before the <svg>
/// tag, but making sure that <svg> is the first tag in the file and that it does exist. When found
/// it obtains the "viewBox" size and scales it by the "scale" factor. Returns a Car object that
/// still holds no polygons but has its dimensions defined.
fn init_svg<'l>(path: &str, scaling: f32, content: &'l mut String) -> Result<(Parser<'l>, Car)> {
    // init car with dummy values
    let mut car = Car {
        polygons: Vec::new(),
        height: 0,
        width: 0,
    };

    let mut parser: Parser = svg::open(path, content)?;

    // Ignoramos las cosas antes de <svg>, pero si no se encuentra <svg> so si se encuentra otra
    // etiqueta antes retornamos error.
    for event in &mut parser {
        match event {
            // Encontramos tag <svg>
            Event::Tag(SVG, Type::Start, attributes) => {
                println!("viewbox: {:?}", attributes.get("viewBox"));

                // parseamos el viewbox
                let viewbox = attributes
                    .get("viewBox")
                    .ok_or_else(|| anyhow!("svg no trae atributo \"viewBox\""))?;

                let viewbox: Vec<f32> = viewbox
                    .split(' ')
                    .map(|s| s.trim().parse::<f32>())
                    .collect::<Result<Vec<f32>, _>>()
                    .with_context(|| {
                        format!(
                            "Valores de viewBox no se pudieron parsear a f32. Falló en: '{}'",
                            viewbox
                        )
                    })?;

                car.width = (viewbox[2] * scaling).round() as u32;
                car.height = (viewbox[3] * scaling).round() as u32;

                return Ok((parser, car));
            }
            // Si encontramos un <tag> que no es svg
            Event::Tag(tag, _, _) => {
                return Err(anyhow!(
                    "Archivo .svg no comienza con etiqueta <svg>, sino '{}'",
                    tag
                ));
            }

            // Otras varas no importan
            other => {
                println!("Antes de svg: {:?}", other);
            }
        }
    }
    // Si nos quedamos sin elementos
    Err(anyhow!("No se encontró elemento <svg>"))
}
