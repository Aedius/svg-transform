use clap::Parser;
use std::any::Any;
use std::f32::MAX;
use std::fs::File;
use std::io::Read;
use svg::node::element::path::{Command, Data, Number, Parameters, Position};
use svg::node::element::tag::{Path, SVG};
use svg::node::element::Path;
use svg::parser::Event;
use svg::parser::Event::Instruction;
use svg::{Document, Node};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    input: String,

    #[clap(short, long)]
    output: String,

    #[clap(short, long)]
    width: Option<u8>,

    #[clap(short, long)]
    length: Option<u8>,
}

fn main() -> Result<(), String> {
    let args: Args = Args::parse();

    println!("{:?}", args);

    let content = get_content(&args.input);

    let mut size = get_size(content.as_str());

    size.prepare(args.width, args.length);

    println!("{:?}", size);

    if !size.is_valid() {
        return Err("cannot get size".to_string());
    }
    let mut output = Document::new();
    for event in svg::read(content.as_str()).unwrap() {
        match event {
            Event::Tag(Path, _, attributes) => {
                let mut new_data = Data::new();

                let data = attributes.get("d").unwrap();
                let data = Data::parse(data).unwrap();
                for command in data.iter() {
                    match command {
                        Command::Move(_, param) => {
                            new_data = new_data.add(Command::Move(
                                Position::Absolute,
                                size.calc(
                                    param.get(0).unwrap().to_owned(),
                                    param.get(1).unwrap().to_owned(),
                                ),
                            ));
                        }
                        Command::Line(_, param) => {
                            new_data = new_data.add(Command::Line(
                                Position::Absolute,
                                size.calc(
                                    param.get(0).unwrap().to_owned(),
                                    param.get(1).unwrap().to_owned(),
                                ),
                            ));
                        }
                        Command::Close => {
                            new_data = new_data.add(Command::Close);
                        }
                        _ => {
                            panic!("command {:?} not implemented", command)
                        }
                    }
                }

                new_data = new_data.close();

                let path = Path::new()
                    .set(
                        "style",
                        "stroke:none;fill-rule:nonzero;fill:rgb(0, 0, 0);fill-opacity:1;",
                    )
                    .set("d", new_data);
                output.append(path);
            }
            Event::Tag(SVG, _, _) => {
                println!("svg");
            }
            Event::Instruction => {
                println!("instruction");
            }
            _ => {
                panic!("event {:?} not implemented", event)
            }
        }
    }

    if args.width.is_some() && args.length.is_some() {
        output = output.set("viewBox", (0, 0, args.width.unwrap(), args.length.unwrap()));
    } else {
        output = output.set(
            "viewBox",
            (
                0,
                0,
                size.x_length.unwrap().round() as usize,
                size.y_length.unwrap().round() as usize,
            ),
        );
    }

    svg::save(&args.output, &output).unwrap();
    Ok(())
}

fn get_content(path: &str) -> String {
    let mut content = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut content).unwrap();
    content
}

fn get_size(content: &str) -> Size {
    let mut size = Size::default();

    for event in svg::read(&content).unwrap() {
        match event {
            Event::Tag(Path, _, attributes) => {
                let data = attributes.get("d").unwrap();
                let data = Data::parse(data).unwrap();
                for command in data.iter() {
                    match command {
                        Command::Move(position, param) => {
                            if position == &Position::Absolute {
                                size.add_pt(
                                    param.get(0).unwrap().to_owned(),
                                    param.get(1).unwrap().to_owned(),
                                )
                            } else {
                                size.all_absolute = false;
                            }
                        }
                        Command::Line(position, param) => {
                            if position == &Position::Absolute {
                                size.add_pt(
                                    param.get(0).unwrap().to_owned(),
                                    param.get(1).unwrap().to_owned(),
                                )
                            } else {
                                size.all_absolute = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    size
}

#[derive(Debug)]
struct Size {
    x_min: Number,
    x_max: Number,
    y_min: Number,
    y_max: Number,
    all_absolute: bool,
    x_length: Option<Number>,
    y_length: Option<Number>,
    target_x_length: Option<Number>,
    target_y_length: Option<Number>,
}

impl Default for Size {
    fn default() -> Self {
        Size {
            x_min: MAX,
            x_max: 0 as Number,
            y_min: MAX,
            y_max: 0 as Number,
            all_absolute: true,
            x_length: None,
            y_length: None,
            target_x_length: None,
            target_y_length: None,
        }
    }
}

impl Size {
    fn is_valid(&self) -> bool {
        return self.all_absolute;
    }

    fn add_pt(&mut self, x: Number, y: Number) {
        if x < self.x_min {
            self.x_min = x
        }
        if x > self.x_max {
            self.x_max = x
        }
        if y < self.y_min {
            self.y_min = y
        }
        if y > self.y_max {
            self.y_max = y
        }
    }

    fn prepare(&mut self, target_x_length: Option<u8>, target_y_length: Option<u8>) {
        self.x_length = Some(self.x_max - self.x_min);
        self.y_length = Some(self.y_max - self.y_min);
        if target_x_length.is_some() {
            self.target_x_length = Some(target_x_length.unwrap() as Number)
        } else {
            self.target_x_length = self.x_length;
        }
        if target_y_length.is_some() {
            self.target_y_length = Some(target_y_length.unwrap() as Number)
        } else {
            self.target_y_length = self.y_length;
        }
    }

    fn calc(&self, x: Number, y: Number) -> Parameters {
        let sx = x - self.x_min;
        let out_x = sx / self.x_length.unwrap() * self.target_x_length.unwrap() as Number;

        let sy = y - self.y_min;
        let out_y = sy / self.y_length.unwrap() * self.target_y_length.unwrap() as Number;

        Parameters::from((out_x, out_y))
    }
}
