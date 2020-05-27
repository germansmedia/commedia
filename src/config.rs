// (C) Copyright 2020, by Germans Media Technology & Services
// Commedia
// YAML Config file loader

use std::fs::File;
use std::io::{BufRead,BufReader,Lines};

use crate::*;

#[derive(Debug)]
struct Line {
    indent: usize,
    dashed: bool,
    key: String,
    value: String,
}

#[derive(Debug)]
struct Parser<'a> {
    lines: Lines<BufReader<&'a File>>,
    pub linenr: usize,
    pub line: Option<Line>,
}

impl<'a> Parser<'a> {
    pub fn accept(&mut self) {
        loop {
            let mut line = match self.lines.next() {
                None => {
                    self.line = None;
                    return;  // end-of-file
                },
                Some(line) => match line {
                    Err(_) => {
                        self.line = None;
                        return;  // read error
                    },
                    Ok(line) => line,
                },
            };
            self.linenr += 1;
            let mut indent = 0;
            let mut chars = line.chars();
            while let Some(c) = chars.next() {
                if c == ' ' {
                    indent += 1;
                }
                else {
                    break;
                }
            }
            line = line.split_off(indent);
            if !line.is_empty() && !line.starts_with('#') {
                let dashed = if line.starts_with('-') {
                    line = line.split_off(1).trim().to_string();
                    true
                }
                else {
                    false
                };
                let result: Vec<&str> = line.split(':').collect();
                let key = result[0].to_string();
                let value = if result.len() > 1 {
                    result[1..].join(" ").trim().to_string()
                }
                else {
                    "".to_string()
                };
                self.line = Some(Line { indent: indent,dashed: dashed,key: key,value: value, });
                return;
            }
        }
    }        
}

#[derive(Debug)]
pub enum SessionPath {
    Replace(String),
    Append(String),
}

#[derive(Debug)]
pub enum SessionStyle {
    Still,        // still image per instance
    StillDepth,   // still image + depth per instance
    Moving,       // sequences of still images per instance
    MovingDepth,  // sequences of still images + depth per instance
}

#[derive(Debug)]
pub enum SessionFormat {
    BMP,       // as BMP (depth is stored in the alpha channel)
    PNG,       // as PNG (depth is stored in the alpha channel)
    ProtoBuf,  // as TensorFlow protobuf array
}

#[derive(Debug)]
pub enum SessionDistribution {
    Constant(f32),  // value (identical for each instance)
    Normal(rand_distr::Normal<f32>),  // avg,stddev (different by stddev around avg for each instance)
}

impl SessionDistribution {
    pub fn instantiate(&self,rng: &mut rand::rngs::ThreadRng) -> f32 {
        match self {
            SessionDistribution::Constant(value) => *value,
            SessionDistribution::Normal(normal) => normal.sample(rng) as f32,
        }
    }
}

#[derive(Debug)]
pub struct SessionXYZ {
    pub x: SessionDistribution,
    pub y: SessionDistribution,
    pub z: SessionDistribution,
}

#[derive(Debug)]
pub struct SessionYPB {
    pub y: SessionDistribution,
    pub p: SessionDistribution,
    pub b: SessionDistribution,
}

#[derive(Debug)]
pub struct SessionRGB {
    pub r: SessionDistribution,
    pub g: SessionDistribution,
    pub b: SessionDistribution,
}

#[derive(Debug)]
pub enum SessionBackground {
    Color(SessionRGB),  // colored background
    Image(String),  // randomly selected crop of randomly selected image from a directory
}

#[derive(Debug)]
pub struct Session {
    pub name: String,
    pub path: SessionPath,
    pub csv: String,
    pub count: usize,
    pub style: SessionStyle,
    pub format: SessionFormat,
    pub size: usizexy,
    pub projection: f32m4x4,
    pub head_pos: SessionXYZ,
    pub head_dir: SessionYPB,
    pub lefteye: SessionYPB,
    pub righteye: SessionYPB,
    pub light_dir: SessionYPB,
    pub light_color: SessionRGB,
    pub background: SessionBackground,
    pub ambient_color: SessionRGB,
    pub skin_color: SessionRGB,
    pub sclera_color: SessionRGB,
    pub iris_color: SessionRGB,
}

fn parse_distribution(parser: &mut Parser) -> Option<SessionDistribution> {
    if let Some(line) = &parser.line {
        if line.value.starts_with("normal") {
            let value = line.value["normal".len()..].trim().to_string();
            let comp: Vec<&str> = value.split(',').collect();
            if comp.len() != 2 {
                println!("line {}: normal distribution has 2 parameters: avg and stddev",parser.linenr);
                return None;
            }
            let avg = comp[0].parse::<f32>().unwrap();
            let stddev = comp[1].parse::<f32>().unwrap();
            Some(SessionDistribution::Normal(rand_distr::Normal::<f32>::new(avg,stddev).unwrap()))
        }
        else if let Ok(value) = line.value.parse::<f32>() {
            Some(SessionDistribution::Constant(value))
        }
        else {
            println!("line {}: constant or normal distribution expected",parser.linenr);
            None
        }
    }
    else {
        None
    }
}

fn parse_xyz(parser: &mut Parser) -> Option<SessionXYZ> {
    let mut pos = SessionXYZ {
        x: SessionDistribution::Constant(0.0),
        y: SessionDistribution::Constant(0.0),
        z: SessionDistribution::Constant(0.0),
    };
    let current_indent = if let Some(line) = &parser.line {
        line.indent
    }
    else {
        println!("line {}: missing position specification",parser.linenr);
        return None;
    };
    while let Some(line) = &parser.line {
        match line.key.as_str() {
            "x" => {
                pos.x = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "y" => {
                pos.y = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "z" => {
                pos.z = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            _ => {
                if line.indent != current_indent {
                    return Some(pos);
                }
                else {
                    println!("line {}: x, y or z expected",parser.linenr);
                    return None;
                }
            }
        }
    }
    Some(pos)
}

fn parse_ypb(parser: &mut Parser) -> Option<SessionYPB> {
    let mut dir = SessionYPB {
        y: SessionDistribution::Constant(0.0),
        p: SessionDistribution::Constant(0.0),
        b: SessionDistribution::Constant(0.0),
    };
    let current_indent = if let Some(line) = &parser.line {
        line.indent
    }
    else {
        println!("line {}: missing direction specification",parser.linenr);
        return None;
    };
    while let Some(line) = &parser.line {
        match line.key.as_str() {
            "y" => {
                dir.y = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "p" => {
                dir.p = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "b" => {
                dir.b = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            _ => {
                if line.indent != current_indent {
                    return Some(dir);
                }
                else {
                    println!("line {}: y, p or b expected",parser.linenr);
                    return None;
                }
            }
        }
    }
    Some(dir)
}

fn parse_rgb(parser: &mut Parser) -> Option<SessionRGB> {
    let mut color = SessionRGB {
        r: SessionDistribution::Constant(0.0),
        g: SessionDistribution::Constant(0.0),
        b: SessionDistribution::Constant(0.0),
    };
    let current_indent = if let Some(line) = &parser.line {
        line.indent
    }
    else {
        println!("line {}: missing color specification",parser.linenr);
        return None;
    };
    while let Some(line) = &parser.line {
        match line.key.as_str() {
            "r" => {
                color.r = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "g" => {
                color.g = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            "b" => {
                color.b = if let Some(value) = parse_distribution(parser) { value } else { return None; };
                parser.accept();
            },
            _ => {
                if line.indent != current_indent {
                    return Some(color);
                }
                else {
                    println!("line {}: r, g or b expected",parser.linenr);
                    return None;
                }
            }
        }
    }
    Some(color)
}

fn parse_head(parser: &mut Parser) -> Option<(SessionXYZ,SessionYPB)> {
    let mut pos = SessionXYZ {
        x: SessionDistribution::Constant(0.0),
        y: SessionDistribution::Constant(0.0),
        z: SessionDistribution::Constant(0.0),
    };
    let mut dir = SessionYPB {
        y: SessionDistribution::Constant(0.0),
        p: SessionDistribution::Constant(0.0),
        b: SessionDistribution::Constant(0.0),
    };
    let current_indent = if let Some(line) = &parser.line {
        line.indent
    }
    else {
        println!("line {}: missing head specification",parser.linenr);
        return None;
    };
    while let Some(line) = &parser.line {
        match line.key.as_str() {
            "pos" => {
                parser.accept();
                pos = if let Some(value) = parse_xyz(parser) { value } else { return None; };
            },
            "dir" => {
                parser.accept();
                dir = if let Some(value) = parse_ypb(parser) { value } else { return None; };
            },
            _ => {
                if line.indent != current_indent {
                    return Some((pos,dir))
                }
                else {
                    println!("line {}: pos or dir expected",parser.linenr);
                    return None;
                }
            },
        }
    }
    Some((pos,dir))
}

fn parse_light(parser: &mut Parser) -> Option<(SessionYPB,SessionRGB)> {
    let mut dir = SessionYPB {
        y: SessionDistribution::Constant(0.0),
        p: SessionDistribution::Constant(0.0),
        b: SessionDistribution::Constant(0.0),
    };
    let mut color = SessionRGB {
        r: SessionDistribution::Constant(0.0),
        g: SessionDistribution::Constant(0.0),
        b: SessionDistribution::Constant(0.0),
    };
    let current_indent = if let Some(line) = &parser.line {
        line.indent
    }
    else {
        println!("line {}: missing light specification",parser.linenr);
        return None;
    };
    while let Some(line) = &parser.line {
        match line.key.as_str() {
            "dir" => {
                parser.accept();
                dir = if let Some(value) = parse_ypb(parser) { value } else { return None; };
            },
            "color" => {
                parser.accept();
                color = if let Some(value) = parse_rgb(parser) { value } else { return None; };
            },
            _ => {
                if line.indent != current_indent {
                    return Some((dir,color))
                }
                else {
                    println!("line {}: dir or color expected",parser.linenr);
                    return None;
                }
            },
        }
    }
    Some((dir,color))
}

pub fn load_config(name: &str) -> Option<Vec<Session>> {
    let file = File::open(name).expect("cannot open config file");
    let reader = BufReader::new(&file);
    let mut parser = Parser {
        lines: reader.lines(),
        linenr: 0usize,
        line: None,
    };
    parser.accept();
    let mut sessions: Vec<Session> = Vec::new();
    while let Some(line) = &parser.line {
        if line.indent != 0 {
            println!("line {}: session should start at first column",parser.linenr);
            return None;
        }
        let mut session = Session {
            name: line.key.clone(),
            path: SessionPath::Replace("./".to_string()),
            csv: "./files.cvs".to_string(),
            count: 16384,
            style: SessionStyle::Still,
            format: SessionFormat::BMP,
            size: usizexy { x: 256,y: 192, },
            projection: f32m4x4::perspective(30.0,4.0 / 3.0,0.1,100.0),
            head_pos: SessionXYZ {
                x: SessionDistribution::Constant(0.0),
                y: SessionDistribution::Constant(0.0),
                z: SessionDistribution::Constant(0.0),
            },
            head_dir: SessionYPB {
                y: SessionDistribution::Constant(0.0),
                p: SessionDistribution::Constant(0.0),
                b: SessionDistribution::Constant(0.0),
            },
            lefteye: SessionYPB {
                y: SessionDistribution::Constant(0.0),
                p: SessionDistribution::Constant(0.0),
                b: SessionDistribution::Constant(0.0),
            },
            righteye: SessionYPB {
                y: SessionDistribution::Constant(0.0),
                p: SessionDistribution::Constant(0.0),
                b: SessionDistribution::Constant(0.0),
            },
            light_dir: SessionYPB {
                y: SessionDistribution::Constant(0.0),
                p: SessionDistribution::Constant(0.0),
                b: SessionDistribution::Constant(0.0),    
            },
            light_color: SessionRGB {
                r: SessionDistribution::Constant(1.0),
                g: SessionDistribution::Constant(1.0),
                b: SessionDistribution::Constant(1.0),
            },
            background: SessionBackground::Color(SessionRGB {
                r: SessionDistribution::Constant(0.0),
                g: SessionDistribution::Constant(0.0),
                b: SessionDistribution::Constant(0.0),
            }),
            ambient_color: SessionRGB {
                r: SessionDistribution::Constant(0.2),
                g: SessionDistribution::Constant(0.2),
                b: SessionDistribution::Constant(0.2),
            },
            skin_color: SessionRGB {
                r: SessionDistribution::Constant(0.8),
                g: SessionDistribution::Constant(0.7),
                b: SessionDistribution::Constant(0.6),
            },
            sclera_color: SessionRGB {
                r: SessionDistribution::Constant(0.8),
                g: SessionDistribution::Constant(0.8),
                b: SessionDistribution::Constant(0.8),
            },
            iris_color: SessionRGB {
                r: SessionDistribution::Constant(0.2),
                g: SessionDistribution::Constant(0.3),
                b: SessionDistribution::Constant(0.4),
            },
        };
        parser.accept();
        while let Some(line) = &parser.line {
            match line.key.as_str() {
                "path" => {
                    session.path = if line.value.starts_with("replace") {
                        SessionPath::Replace(line.value["replace".len()..].trim().to_string())
                    }
                    else if line.value.starts_with("append") {
                        SessionPath::Append(line.value["append".len()..].trim().to_string())
                    }
                    else {
                        println!("line {}: replace or append expected",parser.linenr);
                        return None;
                    };
                    parser.accept();
                },
                "csv" => {
                    session.csv = line.value.clone();
                    parser.accept();
                },
                "count" => {
                    session.count = line.value.parse::<usize>().unwrap();
                    parser.accept();
                },
                "style" => {
                    session.style = match line.value.as_str() {
                        "still" => SessionStyle::Still,
                        "still_depth" => SessionStyle::StillDepth,
                        "moving" => SessionStyle::Moving,
                        "moving_depth" => SessionStyle::MovingDepth,
                        _ => {
                            println!("line {}: invalid session style (should be still, still_depth, moving or moving_depth)",parser.linenr);
                            return None;
                        },
                    };
                    parser.accept();
                },
                "format" => {
                    session.format = match line.value.as_str() {
                        "bmp" => SessionFormat::BMP,
                        "png" => SessionFormat::PNG,
                        "protobuf" => SessionFormat::ProtoBuf,
                        _ => {
                            println!("line {}: invalid session format (should be bmp, png or protobuf)",parser.linenr);
                            return None;
                        },
                    };
                    parser.accept();
                },
                "size" => {
                    let comp: Vec<&str> = line.value.split(',').collect();
                    session.size.x = comp[0].parse::<usize>().unwrap();
                    session.size.y = comp[1].parse::<usize>().unwrap();
                    parser.accept();
                },
                "projection" => {
                    session.projection = if line.value.starts_with("perspective") {
                        let value = line.value["perspective".len()..].trim().to_string();
                        let comp: Vec<&str> = value.split(',').collect();
                        if comp.len() != 4 {
                            println!("line {}: perspective has 4 parameters: fovy, aspect, near and far",parser.linenr);
                            return None;
                        }
                        let fovy = comp[0].parse::<f32>().unwrap();
                        let aspect = if comp[1].contains('/') {
                            let vals: Vec<&str> = comp[1].split('/').collect();
                            let num = vals[0].parse::<f32>().unwrap();
                            let den = vals[1].parse::<f32>().unwrap();
                            num / den
                        }
                        else {
                            comp[1].parse::<f32>().unwrap()
                        };
                        let near = comp[2].parse::<f32>().unwrap();
                        let far = comp[3].parse::<f32>().unwrap();
                        f32m4x4::perspective(fovy,aspect,near,far)
                    }
                    else {
                        println!("line {}: only perspective supported",parser.linenr);
                        return None;
                    };
                    parser.accept();
                },
                "head" => {
                    parser.accept();
                    let result = if let Some((pos,dir)) = parse_head(&mut parser) { (pos,dir) } else { return None; };
                    session.head_pos = result.0;
                    session.head_dir = result.1;
                },
                "lefteye" => {
                    parser.accept();
                    session.lefteye = if let Some(value) = parse_ypb(&mut parser) { value } else { return None; };
                },
                "righteye" => {
                    parser.accept();
                    session.righteye = if let Some(value) = parse_ypb(&mut parser) { value } else { return None; };
                },
                "light" => {
                    parser.accept();
                    let result = if let Some((dir,color)) = parse_light(&mut parser) { (dir,color) } else { return None; };
                    session.light_dir = result.0;
                    session.light_color = result.1;
                },
                "background" => {
                    session.background = match line.value.as_str() {
                        "black" => {
                            SessionBackground::Color(SessionRGB {
                                r: SessionDistribution::Constant(0.0),
                                g: SessionDistribution::Constant(0.0),
                                b: SessionDistribution::Constant(0.0),
                            })
                        },
                        "color" => {
                            SessionBackground::Color(if let Some(value) = parse_rgb(&mut parser) { value } else { return None; })
                        },
                        _ => {
                            if line.value.starts_with("image") {
                                SessionBackground::Image(line.value["image".len()..].trim().to_string())
                            }
                            else {
                                println!("line {}: expected black, color or image",parser.linenr);
                                return None;
                            }
                        }
                    };
                    parser.accept();
                },
                "ambient" => {
                    parser.accept();
                    session.ambient_color = if let Some(value) = parse_rgb(&mut parser) { value } else { return None; };
                },
                "skin" => {
                    parser.accept();
                    session.skin_color = if let Some(value) = parse_rgb(&mut parser) { value } else { return None; };
                },
                "sclera" => {
                    parser.accept();
                    session.sclera_color = if let Some(value) = parse_rgb(&mut parser) { value } else { return None; };
                },
                "iris" => {
                    parser.accept();
                    session.iris_color = if let Some(value) = parse_rgb(&mut parser) { value } else { return None; };
                },
                _ => {
                    println!("line {}: invalid key {}",parser.linenr,line.key);
                    return None;
                }
            }
        }
        sessions.push(session);
    }
    Some(sessions)
}
