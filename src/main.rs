use svg::node::element::path::{Command, Data};
use svg::node::element::tag::{Path, Type};
use svg::parser::Event;

fn main() {
    let path = "images/car.svg";
    let mut content = String::new();
    for event in svg::open(path, &mut content).unwrap() {
        match event {
            Event::Tag(Path, Type::Empty | Type::Start, attributes) => {
                println!("Path id: {}", attributes.get("id").unwrap());
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
            _ => {}
        }
    }
}
