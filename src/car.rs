#![allow(non_upper_case_globals)]

use std::io;

use svg::node::element::path::{Command, Data};
use svg::node::element::tag::{Circle, Ellipse, Group, Path, Type, SVG};
use svg::parser::Event;

use crate::shapes::Polygon;

pub struct Car {
    polygons: Vec<Polygon>,
    height: u32,
    width: u32,
}

pub fn parse_svg(path: &str, scaling: f32, distance: f32) -> io::Result<Car> {
    // init car with dummy values
    let mut car = Car {
        polygons: Vec::new(),
        height: 0,
        width: 0,
    };

    let mut content = String::new();
    let mut svg = svg::open(path, &mut content)?;

    // Ignoramos las cosas antes de <svg>, pero si no se encuentra <svg> so si se encuentra otra
    // etiqueta antes retornamos error.
    loop {
        match svg.next() {
            // Encontramos tag <svg>
            Some(Event::Tag(SVG, Type::Start, attributes)) => {
                println!("viewbox: {:?}", attributes.get("viewBox"));

                // parseamos el viewbox
                let viewbox: Vec<f32> = match attributes
                    .get("viewBox")
                    .expect("svg no trae atributo \"viewBox\"")
                    .split(' ')
                    .map(|s| s.trim().parse::<f32>())
                    .collect::<Result<Vec<f32>, _>>()
                {
                    Ok(vec) => vec,
                    Err(e) => {
                        println!("Error parseando valores de viewBox: {}", e);
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Valores de viewBox no se pudieron parsear a f32",
                        ));
                    }
                };

                car.width = (viewbox[2] * scaling).round() as u32;
                car.height = (viewbox[3] * scaling).round() as u32;

                break;
            }
            // Si encontramos un <tag> que no es svg
            Some(Event::Tag(tag, _, _)) => {
                println!(
                    "Primer elemento de svg no es una etiqueta svg, sino {}",
                    tag
                );
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Svg buscado no comienza con etiquta <svg>",
                ));
            }

            // Si nos quedamos sin elementos
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No se encontró elemento <svg>",
                ));
            }

            // Otras varas no importan
            Some(other) => {
                println!("Antes de svg: {:?}", other);
            }
        }
    }

    let mut layer: i32 = 0;

    for event in svg {
        match event {
            // Group = layers
            Event::Tag(Group, Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .expect("group no trae id (layer sin número)");
                println!("Layer: '{}'", id);
                layer = match id.parse() {
                    Ok(l) => l,
                    Err(e) => {
                        println!("Error parseando valores de viewBox: {}", e);
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Valores de viewBox no se pudieron parsear a f32",
                        ));
                    }
                }
            }

            // Path = líneas/curvas
            Event::Tag(Path, Type::Empty | Type::Start, attributes) => {
                let id = attributes.get("id").expect("path no trae id");
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
                let id = attributes.get("id").expect("circle no trae id");
                println!("Circle id: {}", id);
            }
            Event::Tag(Ellipse, Type::Empty | Type::Start, attributes) => {
                let id = attributes.get("id").expect("ellipse no trae id");
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
