// TODO: get rid of 'as' casting

use anyhow::{anyhow, Context, Result};
//Err(anyhow!(
//    "Esta función está incompleta y no se debe llamar: 'parse_color()'"
//))
use impls::impls;
use itertools::Itertools;
use std::collections::HashMap;
use std::f32::consts::PI;
use svg::node::element::{
    path::{Command, Data, Parameters, Position},
    tag::{self, Type},
};
use svg::node::Attributes;
use svg::parser::{Event, Parser};

use crate::constants::POLYLINE_N;
use crate::shapes::{Color, Line, LineMethods, Point, Polygon, Universal};

pub type Car = Vec<Polygon<Universal>>;

/// Function made to specifically parse the "car.svg" file and return a "Car" object (which is just
/// Vec<Polygon>.
pub fn parse_svg(path: &str, scene_size: u32, distance: Universal) -> Result<Car> {
    if !impls!(Universal: From<f32>) {
        return Err(anyhow!("tipo Universal no es convertible a f32"));
    }

    let mut content = String::new();
    let (parser, mut car, scaling) = init_svg::<f32>(path, scene_size, &mut content)?;

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
                let poly_path = approximate_path(attributes, layer, scaling, distance)?;
                car.push(poly_path);
            }
            Event::Tag(tag::Circle, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("circle no trae id"))?;
                println!("Circle id: {}", id);
                let poly_circle = approximate_circle(attributes, layer, scaling, distance)?;
                car.push(poly_circle);
            }
            Event::Tag(tag::Ellipse, Type::Empty | Type::Start, attributes) => {
                let id = attributes
                    .get("id")
                    .ok_or_else(|| anyhow!("ellipse no trae id"))?;
                println!("Ellipse id: {}", id);
                let poly_ellipse = approximate_ellipse(attributes, layer, scaling, distance)?;
                car.push(poly_ellipse);
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

    println!(
        "Car was approximated using {} vertices! And {} polygons",
        count_vertices(&car),
        car.len(),
    );

    Ok(car)
}

pub fn count_vertices(car: &Car) -> usize {
    car.iter().flat_map(|p| p.get_borders()).flatten().count()
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

fn init_polygon(attributes: &Attributes, layer: i32) -> Result<Polygon<Universal>> {
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
    poly.set_fill_color(style.fill);
    Ok(poly)
}

fn approx_cubic_bezier_aux(
    segment: &[Universal],
    anchor: Point<Universal>,
    n: u32,
) -> Result<Line<Universal>> {
    let p0 = anchor;
    let p1 = Point::new_unchecked(segment[0] + p0.x(), segment[1] + p0.y());
    let p2 = Point::new_unchecked(segment[2] + p0.x(), segment[3] + p0.y());
    let p3 = Point::new_unchecked(segment[4] + p0.x(), segment[5] + p0.y());
    //println!(
    //    "Approximating curve:\n\tp0: {:?}\tp1: {:?}\n\tp2: {:?}\tp3: {:?}",
    //    p0, p1, p2, p3
    //);
    let b = |t: Universal| {
        Point::<Universal>::new(
            (1.0 - t).powi(3) * p0.x()
                + 3.0 * (1.0 - t).powi(2) * t * p1.x()
                + 3.0 * (1.0 - t) * t * t * p2.x()
                + t.powi(3) * p3.x(),
            (1.0 - t).powi(3) * p0.y()
                + 3.0 * (1.0 - t).powi(2) * t * p1.y()
                + 3.0 * (1.0 - t) * t * t * p2.y()
                + t.powi(3) * p3.y(),
        )
    };

    (0..n)
        .map(|t| b((t as Universal) / n as Universal))
        .collect::<Result<Line<Universal>>>()
}

fn approximate_cubic_beziers(
    points: &Parameters,
    anchor: Point<Universal>,
    distance: Universal,
) -> Result<Line<Universal>> {
    let mut beziers: Vec<Line<Universal>> = Vec::new();
    let mut p0 = anchor;
    for segment in points.chunks(6) {
        let length = approx_cubic_bezier_aux(segment, p0, POLYLINE_N)?.euclidean_length();
        //println!("Length: {}", length);
        let n = length / distance;
        let approximation = approx_cubic_bezier_aux(segment, p0, n.round() as u32)?;
        //println!("Approximation: {:?}", approximation);
        beziers.push(approximation);
        p0 = Point::<Universal>::new(p0.x() + segment[4], p0.y() + segment[5])?;
    }

    Ok(beziers.concat())
}

fn get_anchor(borders: &[Line<Universal>], command: &Command) -> Result<Point<Universal>> {
    Ok(*(borders.last().ok_or_else(|| { anyhow!( "Llamado comando '{:?}' sin haber inicializado algún Line dentro de borders", command) })?
            .last().ok_or_else(|| { anyhow!("Llamado comando '{:?}' sin haber agregado ningún punto previo a último borde (osea sin comando 'm')", command)})?))
}

fn get_last_border_mut<'a>(
    borders: &'a mut Vec<Line<Universal>>,
    command: &Command,
) -> Result<&'a mut Line<Universal>> {
    borders.last_mut().ok_or_else(|| {
        anyhow!(
            "Llamado comando '{:?}' sin haber inicializado algún Line dentro de borders",
            command
        )
    })
}

fn approximate_straight_lines(
    command: &Command,
    borders: &[Line<Universal>],
) -> Result<Line<Universal>> {
    match command {
        l @ Command::Line(Position::Relative, params) => {
            //println!("l command");
            (params.len() % 2 == 0)
                .then(|| ())
                .ok_or_else(|| anyhow!("Parámetros de comando 'l' no son múltiplos de 2"))?;
            let mut anchor = get_anchor(borders, l)?;
            params
                .chunks_exact(2)
                .map(|p| {
                    anchor = Point::<Universal>::new(anchor.x() + p[0], anchor.y() + p[1])?;
                    Ok(anchor)
                })
                .collect::<Result<Line<Universal>>>()
        }
        h @ Command::HorizontalLine(Position::Relative, params) => {
            //println!("h command");
            let mut anchor = get_anchor(borders, h)?;
            params
                .iter()
                .map(|p| {
                    anchor = Point::<Universal>::new(anchor.x() + p, anchor.y())?;
                    Ok(anchor)
                })
                .collect::<Result<Line<Universal>>>()
        }
        v @ Command::VerticalLine(Position::Relative, params) => {
            //println!("v command");
            let mut anchor = get_anchor(borders, v)?;
            params
                .iter()
                .map(|p| {
                    anchor = Point::<Universal>::new(anchor.x(), anchor.y() + p)?;
                    Ok(anchor)
                })
                .collect::<Result<Line<Universal>>>()
        }
        c => {
            return Err(anyhow!(
                "?!?! LINE ERROR: Unhandled command: {:?} (this shouldn't be possible)",
                c
            ))
        }
    }
}

fn approximate_path<T: Into<Universal>>(
    attributes: Attributes,
    layer: i32,
    scaling: T,
    distance: Universal,
) -> Result<Polygon<Universal>> {
    let data = attributes
        .get("d")
        .ok_or_else(|| anyhow!("path sin atributo 'd'"))?;
    let data =
        Data::parse(data).context("En approximate_path() no se pudo parsear el atributo 'd'.")?;

    let mut borders = Vec::<Line<Universal>>::new();

    for command in data.iter() {
        match command {
            m @ Command::Move(Position::Relative, params) => {
                //println!( "m command:\tx: {}\ty: {}\tborders.len()={}", params[0], params[1], borders.len());
                let new_point = match borders.len() {
                    0 => Point::<Universal>::new(params[0], params[1]),
                    _ => {
                        let anchor = get_anchor(&borders, m)?;
                        Point::<Universal>::new(anchor.x() + params[0], anchor.y() + params[1])
                    }
                }?;
                borders.push(vec![new_point]);
                if params.len() > 2 {
                    let mut extension = approximate_straight_lines(
                        &Command::Line(
                            Position::Relative,
                            Parameters::from({
                                let mut vec = Vec::with_capacity(params.len() - 2);
                                vec.extend_from_slice(&params[2..]);
                                vec
                            }),
                        ),
                        &borders,
                    )?;
                    get_last_border_mut(&mut borders, m)?.append(&mut extension);
                }
            }
            line @ (Command::Line(Position::Relative, _)
            | Command::HorizontalLine(Position::Relative, _)
            | Command::VerticalLine(Position::Relative, _)) => {
                let mut extension = approximate_straight_lines(line, &borders)?;
                get_last_border_mut(&mut borders, line)?.append(&mut extension);
            }
            c @ Command::CubicCurve(Position::Relative, params) => {
                //println!("c command");
                let anchor = get_anchor(&borders, c)?;
                get_last_border_mut(&mut borders, c)?
                    .append(&mut approximate_cubic_beziers(params, anchor, distance)?);
            }
            z @ Command::Close => {
                //println!("z command");
                // Recordar que en un borde que se "completa" (meaning it forms a loop) su punto
                // inicial y el final son el mismo punto
                let border_start = *(borders.last().ok_or_else(|| { anyhow!( "Llamado comando '{:?}' sin haber inicializado algún Line dentro de borders", z) })?
                    .first().ok_or_else(|| { anyhow!("Llamado comando '{:?}' sin haber agregado ningún punto previo a último borde (osea sin comando 'm')", z)})?);

                get_last_border_mut(&mut borders, z)?.push(border_start);
            }
            c => return Err(anyhow!("!!!! PATH ERROR: Unhandled command: {:?}", c)),
        }
    }

    let mut path_poly = init_polygon(&attributes, layer)?;
    path_poly.set_borders(borders);

    //println!("Path finished\n");
    path_poly.scale(scaling.into())
}

fn approximate_circle<T: Into<Universal>>(
    attributes: Attributes,
    layer: i32,
    scaling: T,
    distance: Universal,
) -> Result<Polygon<Universal>> {
    let mut circle_poly = init_polygon(&attributes, layer)?;

    let center = Point::<Universal>::new(
        attributes
            .get("cx")
            .ok_or_else(|| anyhow!("circle no trae 'cx'"))?
            .parse::<f32>()?,
        attributes
            .get("cy")
            .ok_or_else(|| anyhow!("circle no trae 'cy'"))?
            .parse::<f32>()?,
    )?;

    let radius: f32 = attributes
        .get("r")
        .ok_or_else(|| anyhow!("circle no trae 'r'"))?
        .parse::<f32>()?;

    let perimeter: f32 = 2.0 * radius * PI;
    let num_points: u32 = (perimeter / distance).round() as u32;
    let theta: f32 = 2.0 * PI / num_points as f32;

    // circles can assume a single border
    let mut border: Line<Universal> = (0..num_points)
        .map(|i| {
            Point::<Universal>::new(
                Universal::from(center.x() + (theta * i as f32).cos() * radius),
                Universal::from(center.y() + (theta * i as f32).sin() * radius),
            )
        })
        .collect::<Result<Line<Universal>>>()?;

    // Agregar punto inical al final para completar círculo
    border.push(border[0]);

    circle_poly.add_border(border);

    circle_poly.scale(scaling.into())
}

fn approximate_ellipse<T: Into<Universal>>(
    attributes: Attributes,
    layer: i32,
    scaling: T,
    distance: Universal,
) -> Result<Polygon<Universal>> {
    let mut ellipse_poly = init_polygon(&attributes, layer)?;

    let center = Point::<Universal>::new(
        attributes
            .get("cx")
            .ok_or_else(|| anyhow!("ellipse no trae 'cx'"))?
            .parse::<f32>()?,
        attributes
            .get("cy")
            .ok_or_else(|| anyhow!("ellipse no trae 'cy'"))?
            .parse::<f32>()?,
    )?;

    let radius_x: f32 = attributes
        .get("rx")
        .ok_or_else(|| anyhow!("ellipse no trae 'rx'"))?
        .parse::<f32>()?;
    let radius_y: f32 = attributes
        .get("ry")
        .ok_or_else(|| anyhow!("ellipse no trae 'ry'"))?
        .parse::<f32>()?;

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

    let mut border: Line<Universal> = Line::new();
    for t in 0..(2.0 * PI / step).trunc() as u32 {
        let theta: f32 = t as f32 * step;
        if num_points as f32 * run / circ >= next_point {
            next_point += distance;
            border.push(Point::<Universal>::new(
                Universal::from(center.x() + (theta).cos() * radius_x),
                Universal::from(center.y() + (theta).sin() * radius_y),
            )?);
        }
        run += dp(theta);
    }

    // Agregar punto inicial al final para aproximar círculo
    border.push(border[0]);

    // println!(
    //     "Ellipse:\n\tcx:{}\tcy:{}\n\trx:{}\try:{}\n\tApprox: {:?}",
    //     center.x(),
    //     center.y(),
    //     radius_x,
    //     radius_y,
    //     border,
    // );

    ellipse_poly.add_border(border);

    ellipse_poly.scale(scaling.into())
}

/// This function parse the initial lines of the "car.svg" file, ignoring anything before the <svg>
/// tag, but making sure that <svg> is the first tag in the file and that it does exist. When found
/// it obtains the "viewBox" size and scales it by the "scale" factor. Returns a Car object that
/// still holds no polygons but has its dimensions defined.
fn init_svg<'l, T: Into<Universal> + From<f32>>(
    path: &str,
    scene_size: u32,
    content: &'l mut String,
) -> Result<(Parser<'l>, Car, T)> {
    // init car with dummy values
    let car: Car = Vec::new();

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
                    let scaling = T::from(scene_size as f32 / viewbox[2]);
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
