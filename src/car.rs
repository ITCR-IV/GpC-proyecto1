#![allow(non_upper_case_globals)]

// TODO: get rid of 'as' casting
// TODO: approximate_ellipse() todo: refactor approximation to be less loopy

use anyhow::{anyhow, Context, Result};
//Err(anyhow!(
//    "Esta función está incompleta y no se debe llamar: 'parse_color()'"
//))
use itertools::Itertools;
use std::collections::HashMap;
use std::f32::consts::PI;
use svg::node::element::{
    path::{Command, Data},
    tag::{self, Type},
};
use svg::node::Attributes;
use svg::parser::{Event, Parser};

use crate::shapes::{Color, Line, Point, Polygon};

pub struct Car {
    polygons: Vec<Polygon>,
    scene_size: u32,
}

/// Function made to specifically parse the "car.svg" file and return a "Car" object.
pub fn parse_svg(path: &str, scene_size: u32, distance: f32) -> Result<Car> {
    let mut content = String::new();
    let (parser, mut car, scaling) = init_svg(path, scene_size, &mut content)?;

    let mut layer: i32 = 0;

    for event in parser {
        match event {
            // Group = layers
            Event::Tag(tag::Group, Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("group no trae id (layer sin número)"))?;
                println!("Layer: '{}'", id);
                layer = id
                    .parse()
                    .context("id de 'group' (layer) no se pudo parsear a i32")?;
            }

            // Path = líneas/curvas
            Event::Tag(tag::Path, Type::Empty | Type::Start, attributes) => {
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
            Event::Tag(tag::Circle, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("circle no trae id"))?;
                println!("Circle id: {}", id);
                let poly_circle = approximate_circle(attributes, layer, scaling, distance)?;
                car.polygons.push(poly_circle);
            }
            Event::Tag(tag::Ellipse, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("ellipse no trae id"))?;
                println!("Ellipse id: {}", id);
                let poly_ellipse = approximate_ellipse(attributes, layer, scaling, distance)?;
                car.polygons.push(poly_ellipse);
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

    Err(anyhow!(
        "Esta función está incompleta y no se debe llamar: 'parse_svg()'"
    ))
}

struct Style {
    stroke: Option<Color>,
    fill: Option<Color>,
}

/// Parses color from a style attribute in the svg, which can either be in the form
fn parse_color(color: &str) -> Result<Option<Color>> {
    match color {
        "none" => Ok(None),
        hex => Ok(Some(Color::from_hex(hex)?)),
    }
}

fn parse_style(style: &str) -> Result<Style> {
    let style: HashMap<String, String> = style
        .split(';')
        .map(|s| {
            s.split(':')
                .map(|s| s.to_string())
                .collect_tuple()
                .ok_or_else(|| anyhow!("No se pudo separar key:value pair: {}", s))
        })
        .collect::<Result<HashMap<String, String>, _>>()
        .with_context(|| {
            format!(
                "No se pudo separar algún key:value pair dentro de atributo de style: {}",
                style
            )
        })?;

    Ok(Style {
        stroke: parse_color(
            style
                .get("stroke")
                .ok_or_else(|| anyhow!("atributo 'style' no trae 'stroke': {:?}", style))?,
        )?,
        fill: parse_color(
            style
                .get("fill")
                .ok_or_else(|| anyhow!("atributo 'style' no trae 'fill': {:?}", style))?,
        )?,
    })
}

fn init_polygon(attributes: &Attributes, layer: i32) -> Result<Polygon> {
    let id = attributes
        .get("id")
        .ok_or_else(|| anyhow!("elemento (path/circle/ellipse) no trae id"))?;
    let mut poly = Polygon::new(layer, id.to_string());

    let style = parse_style(
        attributes
            .get("style")
            .ok_or_else(|| anyhow!("elemento (path/circle/ellipse) no trae style"))?,
    )?;

    poly.set_stroke_color(style.stroke);
    poly.set_fill_color(style.stroke);
    Ok(poly)
}

fn approximate_path() {}
fn approximate_circle(
    attributes: Attributes,
    layer: i32,
    scaling: f32,
    distance: f32,
) -> Result<Polygon> {
    let mut circle_poly = init_polygon(&attributes, layer)?;

    let center = Point::new(
        attributes
            .get("cx")
            .ok_or_else(|| anyhow!("circle no trae 'cx'"))?
            .parse::<f32>()?
            * scaling,
        attributes
            .get("cy")
            .ok_or_else(|| anyhow!("circle no trae 'cy'"))?
            .parse::<f32>()?
            * scaling,
    )?;

    let radius: f32 = attributes
        .get("r")
        .ok_or_else(|| anyhow!("circle no trae 'r'"))?
        .parse::<f32>()?
        * scaling;

    let perimeter: f32 = 2.0 * radius * PI;
    let num_points: u32 = (perimeter / distance).round() as u32;
    let theta: f32 = 2.0 * PI / num_points as f32;

    // circles can assume a single border
    let mut border: Line = (0..num_points)
        .map(|i| {
            Point::new(
                center.x() + (theta * i as f32).cos() * radius,
                center.y() + (theta * i as f32).sin() * radius,
            )
        })
        .collect::<Result<Line>>()?;

    border.push(border[0]);

    circle_poly.add_border(border);

    Ok(circle_poly)
}

fn approximate_ellipse(
    attributes: Attributes,
    layer: i32,
    scaling: f32,
    distance: f32,
) -> Result<Polygon> {
    // TODO: refactor the approximation to use less looping and more iterating

    let mut ellipse_poly = init_polygon(&attributes, layer)?;

    let center = Point::new(
        attributes
            .get("cx")
            .ok_or_else(|| anyhow!("ellipse no trae 'cx'"))?
            .parse::<f32>()?
            * scaling,
        attributes
            .get("cy")
            .ok_or_else(|| anyhow!("ellipse no trae 'cy'"))?
            .parse::<f32>()?
            * scaling,
    )?;

    let radius_x: f32 = attributes
        .get("rx")
        .ok_or_else(|| anyhow!("ellipse no trae 'rx'"))?
        .parse::<f32>()?
        * scaling;
    let radius_y: f32 = attributes
        .get("ry")
        .ok_or_else(|| anyhow!("ellipse no trae 'ry'"))?
        .parse::<f32>()?
        * scaling;

    let perimeter: f32 = 2.0 * PI * ((radius_x.powi(2) + radius_y.powi(2)) / 2.0).sqrt();
    let num_points: u32 = (perimeter / distance).round() as u32;

    // La siguiente sección de código adapta al siguiente pseudocódigo:
    //
    //dp(t) = sqrt( (r1*sin(t))^2 + (r2*cos(t))^2)
    //circ = sum(dp(t), t=0..2*Pi step 0.0001)
    //
    //n = 20
    //
    //nextPoint = 0
    //run = 0.0
    //for t=0..2*Pi step 0.0001
    //    if n*run/circ >= nextPoint then
    //        set point (r1*cos(t), r2*sin(t))
    //        nextPoint = nextPoint + 1
    //    next
    //    run = run + dp(t)
    //next
    //
    // Ref: https://stackoverflow.com/questions/6972331/how-can-i-generate-a-set-of-points-evenly-distributed-along-the-perimeter-of-an

    let dp = |t: f32| ((radius_x * t.sin()).powi(2) + (radius_y * t.cos()).powi(2)).sqrt();
    let step: f32 = 0.0001;
    let circ: f32 = (0..(2.0 * PI / step).trunc() as u32)
        .map(|i| dp(i as f32 * step))
        .sum();

    let mut run: f32 = 0.0;
    let mut next_point = 0.0;

    let mut border = Line::new();
    for t in 0..(2.0 * PI / step).trunc() as u32 {
        let theta: f32 = t as f32 * step;
        if num_points as f32 * run / circ >= next_point {
            next_point += distance;
            border.push(Point::new(
                center.x() + (theta).cos() * radius_x,
                center.y() + (theta).sin() * radius_y,
            )?);
        }
        run += dp(theta);
    }

    border.push(border[0]);

    println!(
        "Ellipse:\n\tcx:{}\tcy:{}\n\trx:{}\try:{}\n\tApprox: {:?}",
        center.x(),
        center.y(),
        radius_x,
        radius_y,
        border,
    );

    ellipse_poly.add_border(border);

    Ok(ellipse_poly)
}

/// This function parse the initial lines of the "car.svg" file, ignoring anything before the <svg>
/// tag, but making sure that <svg> is the first tag in the file and that it does exist. When found
/// it obtains the "viewBox" size and scales it by the "scale" factor. Returns a Car object that
/// still holds no polygons but has its dimensions defined.
fn init_svg<'l>(
    path: &str,
    scene_size: u32,
    content: &'l mut String,
) -> Result<(Parser<'l>, Car, f32)> {
    // init car with dummy values
    let mut car = Car {
        polygons: Vec::new(),
        scene_size,
    };

    let mut parser: Parser = svg::open(path, content)?;

    // Ignoramos las cosas antes de <svg>, pero si no se encuentra <svg> so si se encuentra otra
    // etiqueta antes retornamos error.
    for event in &mut parser {
        match event {
            // Encontramos tag <svg>
            Event::Tag(tag::SVG, Type::Start, attributes) => {
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

                return if viewbox.len() == 4 && viewbox[2] == viewbox[3] {
                    let scaling: f32 = scene_size as f32 / viewbox[2];
                    Ok((parser, car, scaling))
                } else {
                    Err(anyhow!(
                        "viewBox leído de .svg no es cuadrado o tiene más de dos dimensiones: {:?}",
                        viewbox
                    ))
                };
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
