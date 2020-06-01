// (C) Copyright 2020, by Germans Media Technology & Services
// Commedia

use std::fs;
use std::io::prelude::*;

extern crate rand;
use rand::Rng;
use rand::seq::SliceRandom;

extern crate rand_distr;
use rand_distr::*;

extern crate math;
use math::*;

extern crate image;
use image::*;

mod context3d_xcb_glx_opengl45;
use context3d_xcb_glx_opengl45::*;

mod opengl45;
use opengl45::*;

mod face;
use face::*;

mod config;
use config::*;

struct Context {
    _ctx: Context3D,
    framebuffer: Framebuffer<ARGB8>,
    skin: Skin,
    eye: Eye,
}

impl Context {
    pub fn new(size: usizexy) -> Context {
        let ctx = Context3D::new().expect("Unable to create 3D context.");
        let framebuffer = Framebuffer::<ARGB8>::new(4 * size).expect("Unable to create framebuffer object.");
        framebuffer.bind();
        Context {
            _ctx: ctx,
            framebuffer: framebuffer,
            skin: Skin::new(),
            eye: Eye::new(),
        }
    }
}

fn downsample4(image: Image<ARGB8>) -> Image<ARGB8> {
    let mut dst = Image::<ARGB8>::new(image.size / 4);
    for y in 0..image.size.y / 4 {
        for x in 0..image.size.x / 4 {
            let mut r = 0usize;
            let mut g = 0usize;
            let mut b = 0usize;
            let mut a = 0usize;
            for i in 0..4 {
                for k in 0..4 {
                    let pix = image.pixel(usizexy::new(x * 4 + k,y * 4 + i));
                    r += pix.r as usize;
                    g += pix.g as usize;
                    b += pix.b as usize;
                    a += pix.a as usize;
                }
            }
            r /= 16;
            g /= 16;
            b /= 16;
            a /= 16;
            *dst.pixel_mut(usizexy::new(x,y)) = ARGB8::new_rgba(r as u8,g as u8,b as u8,a as u8);
        }
    }
    dst
}

fn crop_upside_down(source: &Image<ARGB8>,r: usizer) -> Image<ARGB8> {
    let mut image = Image::<ARGB8>::new(r.s);
    for y in 0..r.s.y {
        for x in 0..r.s.x {
            *image.pixel_mut(usizexy { x: x,y: r.s.y - y - 1, }) = *source.pixel(r.o + usizexy { x: x,y: y, });
        }
    }
    image
}

enum InstanceBackground {
    Color(f32rgb),
    Image(Image<ARGB8>),
}

struct Instance {
    head_pos: f32xyz,
    head_dir: f32ypb,
    lefteye: f32ypb,
    righteye: f32ypb,
    light_dir: f32ypb,
    light_color: f32rgb,
    background: InstanceBackground,
    ambient_color: f32rgb,
    skin_color: f32rgb,
    sclera_color: f32rgb,
    iris_color: f32rgb,
}

const LEFT_EYE_POS: f32xyz = f32xyz { x: -0.031,y: 0.026,z: 0.023, };
const RIGHT_EYE_POS: f32xyz = f32xyz { x: 0.031,y: 0.026,z: 0.023, };
const EYE_SIZE: f32xyz = f32xyz { x: 0.0115,y: 0.0115,z: 0.0115, };

fn render_full(rng: &mut rand::rngs::ThreadRng,ctx: &Context,session: &Session,instance: &Instance) -> Image<ARGB8> {

    // prepare matrices
    let light_matrix = f32m3x3::yaw(instance.light_dir.y) * f32m3x3::pitch(instance.light_dir.p) * f32m3x3::roll(instance.light_dir.b);
    let light_dir = light_matrix * f32xyz::new(0.0,1.0,0.0);
    let head_matrix = f32m4x4::translate(instance.head_pos) * f32m4x4::yaw(instance.head_dir.y) * f32m4x4::pitch(instance.head_dir.p);
    let lefteye_matrix = f32m4x4::translate(LEFT_EYE_POS) * f32m4x4::yaw(instance.lefteye.y) * f32m4x4::pitch(instance.lefteye.p) * f32m4x4::scale(EYE_SIZE);
    let righteye_matrix = f32m4x4::translate(RIGHT_EYE_POS) * f32m4x4::yaw(instance.righteye.y) * f32m4x4::pitch(instance.righteye.p) * f32m4x4::scale(EYE_SIZE);
    let depth_map = match session.style {
        SessionStyle::Still => {
            f32xy { x: 1.0,y: 0.0, }
        },
        SessionStyle::StillDepth(scale,offset) => {
            f32xy { x: offset,y: scale, }
        },
        SessionStyle::Moving => {
            f32xy { x: 1.0,y: 0.0, }
        },
        SessionStyle::MovingDepth(scale,offset) => {
            f32xy { x: offset,y: scale, }
        },
    };

    // clear or draw background
    match &instance.background {
        InstanceBackground::Color(color) => {
            ctx.framebuffer.bind();
            unsafe {
                gl::ClearColor(color.r,color.g,color.b,1.0);
                gl::ClearDepth(1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
        },
        InstanceBackground::Image(image) => {
            ctx.framebuffer.bind();
            unsafe {
                gl::ClearDepth(1.0);
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }
            ctx.framebuffer.unbind();
            ctx.framebuffer.set(&image);
            ctx.framebuffer.bind();
        },
    }

    // draw eyes and face
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        ctx.eye.render_full(session.projection,head_matrix * lefteye_matrix,light_dir,instance.light_color,instance.ambient_color,instance.sclera_color,instance.iris_color,depth_map);
        ctx.eye.render_full(session.projection,head_matrix * righteye_matrix,light_dir,instance.light_color,instance.ambient_color,instance.sclera_color,instance.iris_color,depth_map);
        ctx.skin.render_full(session.projection,head_matrix,light_dir,instance.light_color,instance.ambient_color,instance.skin_color,depth_map);
        gl::Disable(gl::DEPTH_TEST);
        gl::Finish();
        gl::Flush();
    }
    ctx.framebuffer.unbind();

    ctx.framebuffer.grab()
}

fn render_spec(ctx: &Context,session: &Session,instance: &Instance) -> Image<ARGB8> {

    // prepare matrices
    let head_matrix = f32m4x4::translate(instance.head_pos) * f32m4x4::yaw(instance.head_dir.y) * f32m4x4::pitch(instance.head_dir.p);
    let lefteye_matrix = f32m4x4::translate(LEFT_EYE_POS) * f32m4x4::yaw(instance.lefteye.y) * f32m4x4::pitch(instance.lefteye.p) * f32m4x4::scale(EYE_SIZE);
    let righteye_matrix = f32m4x4::translate(RIGHT_EYE_POS) * f32m4x4::yaw(instance.righteye.y) * f32m4x4::pitch(instance.righteye.p) * f32m4x4::scale(EYE_SIZE);

    ctx.framebuffer.bind();
    unsafe {
        gl::ClearColor(0.0,0.0,0.0,1.0);
        gl::ClearDepth(1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Enable(gl::DEPTH_TEST);
        ctx.eye.render_spec(session.projection,head_matrix * lefteye_matrix);
        ctx.eye.render_spec(session.projection,head_matrix * righteye_matrix);
        ctx.skin.render_spec(session.projection,head_matrix,f32rgb { r: 0.5,g: 0.5,b: 0.5, });
        gl::Disable(gl::DEPTH_TEST);
        gl::Finish();
        gl::Flush();
    }
    ctx.framebuffer.unbind();

    ctx.framebuffer.grab()
}

fn save_image(image: Image<ARGB8>,name: &str) {
    let data = bmp::encode(&image).expect("Unable to encode image as BMP.");
    let mut file = fs::File::create(name).expect("Unable to create file.");
    file.write_all(&data).expect("Unable to write BMP.");
}

fn process(rng: &mut rand::rngs::ThreadRng,ctx: &Context,session: &Session,backgrounds: &Vec<Image<ARGB8>>,csv: &mut fs::File,num: usize) {

    // get image filename and full filename
    let name = match session.format {
        SessionFormat::BMP => format!("{:05}.bmp",num),
        SessionFormat::PNG => format!("{:05}.png",num),
        SessionFormat::ProtoBuf => format!("{:05}.todo",num),
    };
    let full_name = match &session.path {
        SessionPath::Replace(path) => format!("{}/{}",path,name),
        SessionPath::Append(path) => format!("{}/{}",path,name),
    };

    // build instance
    let mut instance = Instance {
        head_pos: f32xyz {
            x: session.head_pos.x.instantiate(rng),
            y: session.head_pos.y.instantiate(rng),
            z: session.head_pos.z.instantiate(rng),
        },
        head_dir: f32ypb {
            y: session.head_dir.y.instantiate(rng),
            p: session.head_dir.p.instantiate(rng),
            b: session.head_dir.b.instantiate(rng),
        },
        lefteye: f32ypb {
            y: session.lefteye.y.instantiate(rng),
            p: session.lefteye.p.instantiate(rng),
            b: session.lefteye.b.instantiate(rng),
        },
        righteye: f32ypb {
            y: session.righteye.y.instantiate(rng),
            p: session.righteye.p.instantiate(rng),
            b: session.righteye.b.instantiate(rng),
        },
        light_dir: f32ypb {
            y: session.light_dir.y.instantiate(rng),
            p: session.light_dir.p.instantiate(rng),
            b: session.light_dir.b.instantiate(rng),
        },
        light_color: f32rgb {
            r: session.light_color.r.instantiate(rng),
            g: session.light_color.g.instantiate(rng),
            b: session.light_color.b.instantiate(rng),
        },
        background: match &session.background {
            SessionBackground::Color(color) => InstanceBackground::Color(f32rgb {
                r: color.r.instantiate(rng),
                g: color.g.instantiate(rng),
                b: color.b.instantiate(rng),
            }),
            SessionBackground::Image(path) => InstanceBackground::Image({
                let mut background = backgrounds.choose(rng).expect("unable to select background from set");
                while (background.size.x < ctx.framebuffer.size.x) || (background.size.y < ctx.framebuffer.size.y) {
                    background = backgrounds.choose(rng).expect("unable to select background from set");
                }
                let cropspace = background.size - ctx.framebuffer.size;
                let pos = usizexy { x: (rng.gen::<f32>() * (cropspace.x as f32)) as usize,y: (rng.gen::<f32>() * (cropspace.y as f32)) as usize, };
                crop_upside_down(background,usizer { o: pos,s: ctx.framebuffer.size, })            
            }),
        },
        ambient_color: f32rgb {
            r: session.ambient_color.r.instantiate(rng),
            g: session.ambient_color.g.instantiate(rng),
            b: session.ambient_color.b.instantiate(rng),
        },
        skin_color: f32rgb {
            r: session.skin_color.r.instantiate(rng),
            g: session.skin_color.g.instantiate(rng),
            b: session.skin_color.b.instantiate(rng),
        },
        sclera_color: f32rgb {
            r: session.sclera_color.r.instantiate(rng),
            g: session.sclera_color.g.instantiate(rng),
            b: session.sclera_color.b.instantiate(rng),
        },
        iris_color: f32rgb {
            r: session.iris_color.r.instantiate(rng),
            g: session.iris_color.g.instantiate(rng),
            b: session.iris_color.b.instantiate(rng),
        },
    };

    // make sure at least 1 pixel of the face is visible
    let mut visible = false;
    while !visible {

        // and also that head_pos is inside the projection frustum
        loop {
            instance.head_pos = f32xyz {
                x: session.head_pos.x.instantiate(rng),
                y: session.head_pos.y.instantiate(rng),
                z: session.head_pos.z.instantiate(rng),
            };
            let pos = session.projection * f32xyzw { x: instance.head_pos.x,y: instance.head_pos.y,z: instance.head_pos.z,w: 1.0, };
            if (pos.x > -pos.w) && (pos.x < pos.w) && (pos.y > -pos.w) && (pos.y < pos.w) {
                break;
            }
        }

        // render specification
        let spec_image = render_spec(&ctx,session,&instance);

        // check image to see if 0.5,0.5,0.5 appeared (face pixels)
        for i in 0..spec_image.size.y {
            for k in 0..spec_image.size.x {
                let r = (*spec_image.pixel(usizexy { x: k,y: i })).r;
                let g = (*spec_image.pixel(usizexy { x: k,y: i })).g;
                let b = (*spec_image.pixel(usizexy { x: k,y: i })).b;
                if (r >= 0x70) && (r < 0x90) && (g >= 0x70) && (g < 0x90) && (b >= 0x70) && (b < 0x90) {
                    visible = true;
                    break;
                }
            }
            if visible {
                break;
            }
        }
    }

    // render final image
    let image = render_full(rng,&ctx,session,&instance);

    // downsample 4x to synthesize subpixel accuracy
    let image = downsample4(image);
    
    // and save the image
    save_image(image,&full_name);

    // calculate NDC coordinates of the head
    let hom = session.projection * f32xyzw { x: instance.head_pos.x,y: instance.head_pos.y,z: instance.head_pos.z,w: 1.0, };
    let ndc = f32xyz {
        x: hom.x / hom.w,
        y: hom.y / hom.w,
        z: hom.z / hom.w,
    };

    // calculate screen coordinates of the head
    let screen = f32xy {
        x: 0.5 * (1.0 + ndc.x) * (session.size.x as f32),
        y: 0.5 * (1.0 - ndc.y) * (session.size.y as f32),
    };

    // write line to CSV file
    let line = format!("\"{}\", {},{},{}, {},{}, {},{},{}, {},{}, {},{}, {},{},{}, {},{},{}, {},{},{}\n",name,
        instance.head_pos.x,instance.head_pos.y,instance.head_pos.z,
        instance.head_dir.y,instance.head_dir.p,
        ndc.x,ndc.y,ndc.z,
        screen.x,screen.y,
        instance.light_dir.y,instance.light_dir.p,
        instance.light_color.r,instance.light_color.g,instance.light_color.b,
        instance.ambient_color.r,instance.ambient_color.g,instance.ambient_color.b,
        instance.skin_color.r,instance.skin_color.g,instance.skin_color.b,
    );
    csv.write_all(line.as_bytes()).expect("Unable to write to CSV file.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage:");
        println!("");
        println!("    commedia <file>");
        println!("");
        println!("where <file> is the config file.");
        return;
    }
    let mut rng = rand::thread_rng();
    let sessions = load_config(&args[1]).expect("unable to load config file");
    for session in sessions {

        println!("session: {}",session.name);

        // read backgrounds, if any
        let mut backgrounds: Vec<Image<ARGB8>> = Vec::new();
        if let SessionBackground::Image(path) = &session.background {
            println!("    loading backgrounds...");
            for entry in fs::read_dir(path).expect("unable to read from backgrounds directory") {
                let entry = entry.expect("invalid entry").file_name().into_string().expect("unable to convert");
                let mut file = fs::File::open(format!("{}{}",path,entry)).expect("cannot open file");
                let mut buffer: Vec<u8> = Vec::new();
                file.read_to_end(&mut buffer).expect("unable to read file");
                let image = decode(&buffer).expect("unable to decode");
                backgrounds.push(image);
            }
        }

        // create context
        let ctx = Context::new(session.size);
        match session.style {
            SessionStyle::Still => {
                println!("    generating {} images",session.count);
            },
            SessionStyle::StillDepth(_scale,_offset) => {
                println!("    generating {} images with depth",session.count);
            },
            SessionStyle::Moving => {
                println!("    generating {} movies",session.count);
            },
            SessionStyle::MovingDepth(_scale,_offset) => {
                println!("    generating {} movies with depth",session.count);
            }
        }

        // create/clear path
        match &session.path {
            SessionPath::Replace(path) => {
                match fs::remove_dir_all(path) { _ => { }, };
                match fs::create_dir(path) { _ => { }, };
            },
            SessionPath::Append(path) => {
                match fs::create_dir(path) { _ => { }, };
            },
        }

        // open CSV
        let mut csv = fs::File::create(&session.csv).expect("unable to create CSV file");

        // main loop
        for i in 0..session.count {
            println!("        {} / {}",i,session.count);
            process(&mut rng,&ctx,&session,&backgrounds,&mut csv,i);
        }        

        // print projection parameters, if any
        println!("    projection matrix:");
        println!("        {:10.7} {:10.7} {:10.7} {:10.7}",session.projection.x.x,session.projection.x.y,session.projection.x.z,session.projection.x.w);
        println!("        {:10.7} {:10.7} {:10.7} {:10.7}",session.projection.y.x,session.projection.y.y,session.projection.y.z,session.projection.y.w);
        println!("        {:10.7} {:10.7} {:10.7} {:10.7}",session.projection.z.x,session.projection.z.y,session.projection.z.z,session.projection.z.w);
        println!("        {:10.7} {:10.7} {:10.7} {:10.7}",session.projection.w.x,session.projection.w.y,session.projection.w.z,session.projection.w.w);
    }
}
