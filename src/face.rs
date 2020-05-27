// (C) Copyright 2020, by Germans Media Technology & Services
// Commedia
// Face rendering

use std::{fs,io,ffi};
use std::io::prelude::*;

use crate::math::*;

use crate::opengl45::*;

pub trait Render {
    fn render();
}

pub struct Skin {
    skin: VertexIndexBuffer<VertexNormal,u16>,
    full_shader: ShaderProgram,
    spec_shader: ShaderProgram,
}

impl Skin {
    pub fn new() -> Skin {
        let file = fs::File::open("face.obj").expect("Unable to open face.obj");
        let reader = io::BufReader::new(file);
        let mut vertices: Vec<VertexNormal> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        for line in reader.lines() {
            if let Ok(line) = line {
                let mut part = line.split_whitespace();
                if let Some(tag) = part.next() {
                    match tag {
                        "o" => { },
                        "v" => {
                            let x = part.next().unwrap().parse::<f32>().unwrap();
                            let y = part.next().unwrap().parse::<f32>().unwrap();
                            let z = part.next().unwrap().parse::<f32>().unwrap();
                            vertices.push(VertexNormal {
                                v: f32xyz::new(x,y,z),
                                n: f32xyz::new(0.0,0.0,1.0),
                            });
                        },
                        "f" => {
                            let i0 = part.next().unwrap().parse::<u16>().unwrap() - 1;
                            let i1 = part.next().unwrap().parse::<u16>().unwrap() - 1;
                            let i2 = part.next().unwrap().parse::<u16>().unwrap() - 1;
                            indices.push(i0);
                            indices.push(i1);
                            indices.push(i2);
                        },
                        _ => {
                            println!("unknown tag: {}",tag);
                        }
                    }
                }
            }
        }
        let mut normals: Vec<f32xyz> = Vec::new();
        for i in 0..indices.len() / 3 {
            let a = indices[i * 3];
            let b = indices[i * 3 + 1];
            let c = indices[i * 3 + 2];
            let ab = vertices[b as usize].v - vertices[a as usize].v;
            let ac = vertices[c as usize].v - vertices[a as usize].v;
            normals.push(f32xyz::cross(ab,ac).norm());
        }
        for i in 0..vertices.len() {
            let mut n = f32xyz { x: 0.0,y: 0.0,z: 0.0, };
            let mut total = 0.0f32;
            for k in 0..indices.len() / 3 {
                let i0 = indices[k * 3] as usize;
                let i1 = indices[k * 3 + 1] as usize;
                let i2 = indices[k * 3 + 2] as usize;
                if (i0 == i) || (i1 == i) || (i2 == i) {
                    n += normals[k];
                    total += 1.0;
                }
            }
            vertices[i].n = n / total;
        }
        let vs = VertexShader::new(r#"
            #version 420 core

            uniform mat4 u_projection;
            uniform mat4 u_modelview;
            uniform mat3 u_normal;

            layout(location = 0) in vec3 i_pos;
            layout(location = 1) in vec3 i_normal;

            out vec3 v_pos;
            out vec3 v_normal;

            void main() {
                v_pos = (u_modelview * vec4(i_pos,1.0)).xyz;
                v_normal = u_normal * i_normal;
                gl_Position = u_projection * vec4(v_pos,1.0);
            }
        "#).expect("Unable to create vertex shader.");
        let fs_full = FragmentShader::new(r#"
            #version 420 core

            uniform vec4 u_ambient_color;
            uniform vec4 u_skin_color;
            uniform vec4 u_light_color;
            uniform vec3 u_light_dir;

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                vec3 n = normalize(v_normal);
                vec3 pu = -v_pos;
                vec3 pun = normalize(pu);
                vec3 rn = reflect(-u_light_dir,n);
                float s = max(pow(dot(rn,pun),16.0),0.0);
                float d = clamp(dot(n,u_light_dir),0.0,1.0);
                vec3 res = u_ambient_color.xyz * u_skin_color.xyz + d * u_light_color.xyz * u_skin_color.xyz + 0.4 * s * u_light_color.xyz;
                o_frag = vec4(res,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        let fs_spec = FragmentShader::new(r#"
            #version 420 core

            uniform vec4 u_skin_color;

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                o_frag = u_skin_color;
            }
        "#).expect("Unable to create fragment shader.");
        Skin {
            skin: VertexIndexBuffer::from(vertices,indices),
            full_shader: ShaderProgram::new(&vs,None,&fs_full).expect("Unable to create skin shader program."),
            spec_shader: ShaderProgram::new(&vs,None,&fs_spec).expect("Unable to create skin shader program."),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,light_dir: f32xyz,light_color: f32rgb,ambient_color: f32rgb,skin_color: f32rgb) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.full_shader.bind();
            self.full_shader.set_uniform("u_projection",projection_matrix);
            self.full_shader.set_uniform("u_modelview",modelview_matrix);
            self.full_shader.set_uniform("u_normal",normal_matrix);
            self.full_shader.set_uniform("u_ambient_color",ambient_color);
            self.full_shader.set_uniform("u_skin_color",skin_color);
            self.full_shader.set_uniform("u_light_color",light_color);
            self.full_shader.set_uniform("u_light_dir",light_dir);
            self.skin.bind();
            gl::DrawElements(gl::TRIANGLES,self.skin.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }

    pub fn render_spec(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,skin_color: f32rgb) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.spec_shader.bind();
            self.spec_shader.set_uniform("u_projection",projection_matrix);
            self.spec_shader.set_uniform("u_modelview",modelview_matrix);
            self.spec_shader.set_uniform("u_normal",normal_matrix);
            self.spec_shader.set_uniform("u_skin_color",skin_color);
            self.skin.bind();
            gl::DrawElements(gl::TRIANGLES,self.skin.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }
}

pub struct Sclera {
    sclera: VertexIndexBuffer<VertexNormal,u16>,
    full_shader: ShaderProgram,
    spec_shader: ShaderProgram,
}

impl Sclera {
    pub fn new() -> Sclera {
        let mut vertices: Vec<VertexNormal> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        for i in 0..9 {
            let phi: f32 = (((i as f32) - 4.0) * TAU) / 24.0;
            for k in 0..32 {
                let theta: f32 = ((k as f32) * TAU) / 32.0;
                let unit = f32xyz { x: (phi as f32).cos() * theta.cos(),y: phi.cos() * theta.sin(),z: phi.sin(), };
                vertices.push(VertexNormal { v: unit,n: unit });
            }
        }
        for i in 0..8 {
            for k in 0..31 {
                indices.push(i * 32 + k);
                indices.push(i * 32 + k + 1);
                indices.push(i * 32 + k + 32);
                indices.push(i * 32 + k + 32);
                indices.push(i * 32 + k + 1);
                indices.push(i * 32 + k + 33);
            }
            indices.push(i * 32 + 31);
            indices.push(i * 32);
            indices.push(i * 32 + 63);
            indices.push(i * 32 + 63);
            indices.push(i * 32);
            indices.push(i * 32 + 32);
        }
        let vs = VertexShader::new(r#"
            #version 420 core

            uniform mat4 u_projection;
            uniform mat4 u_modelview;
            uniform mat3 u_normal;

            layout(location = 0) in vec3 i_pos;
            layout(location = 1) in vec3 i_normal;

            out vec3 v_pos;
            out vec3 v_normal;

            void main() {
                v_pos = (u_modelview * vec4(i_pos,1.0)).xyz;
                v_normal = u_normal * i_normal;
                gl_Position = u_projection * vec4(v_pos,1.0);
            }
        "#).expect("Unable to create vertex shader.");
        let fs_full = FragmentShader::new(r#"
            #version 420 core

            uniform vec4 u_ambient_color;
            uniform vec4 u_sclera_color;
            uniform vec4 u_light_color;
            uniform vec3 u_light_dir;

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                vec3 n = normalize(v_normal);
                vec3 pu = -v_pos;
                vec3 pun = normalize(pu);
                vec3 rn = reflect(-u_light_dir,n);
                float s = max(pow(dot(rn,pun),64.0),0.0);
                float d = clamp(dot(n,u_light_dir),0.0,1.0);
                vec3 res = u_ambient_color.xyz * u_sclera_color.xyz + d * u_light_color.xyz * u_sclera_color.xyz;
                o_frag = vec4(res,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        let fs_spec = FragmentShader::new(r#"
            #version 420 core

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                o_frag = vec4(1.0,1.0,1.0,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        Sclera {
            sclera: VertexIndexBuffer::from(vertices,indices),
            full_shader: ShaderProgram::new(&vs,None,&fs_full).expect("Unable to create eye shader program."),
            spec_shader: ShaderProgram::new(&vs,None,&fs_spec).expect("Unable to create eye shader program."),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,light_dir: f32xyz,light_color: f32rgb,ambient_color: f32rgb,sclera_color: f32rgb) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.full_shader.bind();
            self.full_shader.set_uniform("u_projection",projection_matrix);
            self.full_shader.set_uniform("u_modelview",modelview_matrix);
            self.full_shader.set_uniform("u_normal",normal_matrix);
            self.full_shader.set_uniform("u_ambient_color",ambient_color);
            self.full_shader.set_uniform("u_sclera_color",sclera_color);
            self.full_shader.set_uniform("u_light_color",light_color);
            self.full_shader.set_uniform("u_light_dir",light_dir);
            self.sclera.bind();
            gl::DrawElements(gl::TRIANGLES,self.sclera.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }

    pub fn render_spec(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.spec_shader.bind();
            self.spec_shader.set_uniform("u_projection",projection_matrix);
            self.spec_shader.set_uniform("u_modelview",modelview_matrix);
            self.spec_shader.set_uniform("u_normal",normal_matrix);
            self.sclera.bind();
            gl::DrawElements(gl::TRIANGLES,self.sclera.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }
}

pub struct Iris {
    iris: VertexIndexBuffer<VertexNormal,u16>,
    full_shader: ShaderProgram,
    spec_shader: ShaderProgram,
}

impl Iris {
    pub fn new() -> Iris {
        let mut vertices: Vec<VertexNormal> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let opening: f32 = TAU / 12.0;
        let dist: f32 = opening.cos();
        let outer: f32 = opening.sin();
        let inner: f32 = 0.4 * outer;  // iris radius is between outer and inner, inside inner is the pupil
        for k in 0..32 {
            let theta: f32 = ((k as f32) * TAU) / 32.0;
            let vo = f32xyz { x: outer * theta.cos(),y: outer * theta.sin(),z: dist, };
            let vi = f32xyz { x: inner * theta.cos(),y: inner * theta.sin(),z: dist, };
            vertices.push(VertexNormal { v: vi,n: f32xyz { x: 0.0,y: 0.0,z: 1.0, }});
            vertices.push(VertexNormal { v: vo,n: f32xyz { x: 0.0,y: 0.0,z: 1.0, }});
        }
        for k in 0..31 {
            indices.push(k * 2);
            indices.push(k * 2 + 2);
            indices.push(k * 2 + 1);
            indices.push(k * 2 + 1);
            indices.push(k * 2 + 2);
            indices.push(k * 2 + 3);
        }
        indices.push(62);
        indices.push(0);
        indices.push(63);
        indices.push(63);
        indices.push(0);
        indices.push(1);
        let vs = VertexShader::new(r#"
            #version 420 core

            uniform mat4 u_projection;
            uniform mat4 u_modelview;
            uniform mat3 u_normal;

            layout(location = 0) in vec3 i_pos;
            layout(location = 1) in vec3 i_normal;

            out vec3 v_pos;
            out vec3 v_normal;

            void main() {
                v_pos = (u_modelview * vec4(i_pos,1.0)).xyz;
                v_normal = u_normal * i_normal;
                gl_Position = u_projection * vec4(v_pos,1.0);
            }
        "#).expect("Unable to create vertex shader.");
        let fs_full = FragmentShader::new(r#"
            #version 420 core

            uniform vec4 u_ambient_color;
            uniform vec4 u_iris_color;
            uniform vec4 u_light_color;
            uniform vec3 u_light_dir;

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                vec3 n = normalize(v_normal);
                float d = clamp(dot(n,u_light_dir),0.0,1.0);
                vec3 res = u_ambient_color.xyz * u_iris_color.xyz + d * u_light_color.xyz * u_iris_color.xyz;
                o_frag = vec4(res,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        let fs_spec = FragmentShader::new(r#"
            #version 420 core

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                o_frag = vec4(1.0,1.0,1.0,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        Iris {
            iris: VertexIndexBuffer::from(vertices,indices),
            full_shader: ShaderProgram::new(&vs,None,&fs_full).expect("Unable to create eye shader program."),
            spec_shader: ShaderProgram::new(&vs,None,&fs_spec).expect("Unable to create eye shader program."),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,light_dir: f32xyz,light_color: f32rgb,ambient_color: f32rgb,iris_color: f32rgb) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.full_shader.bind();
            self.full_shader.set_uniform("u_projection",projection_matrix);
            self.full_shader.set_uniform("u_modelview",modelview_matrix);
            self.full_shader.set_uniform("u_normal",normal_matrix);
            self.full_shader.set_uniform("u_ambient_color",ambient_color);
            self.full_shader.set_uniform("u_iris_color",iris_color);
            self.full_shader.set_uniform("u_light_color",light_color);
            self.full_shader.set_uniform("u_light_dir",light_dir);
            self.iris.bind();
            gl::DrawElements(gl::TRIANGLES,self.iris.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }

    pub fn render_spec(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.spec_shader.bind();
            self.spec_shader.set_uniform("u_projection",projection_matrix);
            self.spec_shader.set_uniform("u_modelview",modelview_matrix);
            self.spec_shader.set_uniform("u_normal",normal_matrix);
            self.iris.bind();
            gl::DrawElements(gl::TRIANGLES,self.iris.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }
}

pub struct Pupil {
    pupil: VertexBuffer<f32xyz>,
    full_shader: ShaderProgram,
    spec_shader: ShaderProgram,
}

impl Pupil {
    pub fn new() -> Pupil {
        let mut vertices: Vec<f32xyz> = Vec::new();
        let opening: f32 = TAU / 12.0;
        let dist: f32 = opening.cos();
        let outer: f32 = opening.sin();
        let inner: f32 = 0.4 * outer;
        for k in 0..32 {
            let theta: f32 = ((k as f32) * TAU) / 32.0;
            let vi = f32xyz { x: inner * theta.cos(),y: inner * theta.sin(),z: dist, };
            vertices.push(vi);
        }
        let vs = VertexShader::new(r#"
            #version 420 core

            uniform mat4 u_projection;
            uniform mat4 u_modelview;

            layout(location = 0) in vec3 i_pos;

            out vec3 v_pos;

            void main() {
                v_pos = (u_modelview * vec4(i_pos,1.0)).xyz;
                gl_Position = u_projection * vec4(v_pos,1.0);
            }
        "#).expect("Unable to create vertex shader.");
        let fs_full = FragmentShader::new(r#"
            #version 420 core

            in vec3 v_pos;

            out vec4 o_frag;

            void main(void)
            {
                o_frag = vec4(0.0,0.0,0.0,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        let fs_spec = FragmentShader::new(r#"
            #version 420 core

            in vec3 v_pos;

            out vec4 o_frag;

            void main(void)
            {
                o_frag = vec4(1.0,1.0,1.0,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        Pupil {
            pupil: VertexBuffer::from(vertices),
            full_shader: ShaderProgram::new(&vs,None,&fs_full).expect("Unable to create eye shader program."),
            spec_shader: ShaderProgram::new(&vs,None,&fs_spec).expect("Unable to create eye shader program."),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4) {
        unsafe {
            self.full_shader.bind();
            self.full_shader.set_uniform("u_projection",projection_matrix);
            self.full_shader.set_uniform("u_modelview",modelview_matrix);
            self.pupil.bind();
            gl::DrawArrays(gl::TRIANGLE_FAN,0,self.pupil.vertices() as i32);
        }        
    }

    pub fn render_spec(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4) {
        unsafe {
            self.spec_shader.bind();
            self.spec_shader.set_uniform("u_projection",projection_matrix);
            self.spec_shader.set_uniform("u_modelview",modelview_matrix);
            self.pupil.bind();
            gl::DrawArrays(gl::TRIANGLE_FAN,0,self.pupil.vertices() as i32);
        }        
    }
}

pub struct Cornea {
    cornea: VertexIndexBuffer<VertexNormal,u16>,
    shader: ShaderProgram,
}

impl Cornea {
    pub fn new() -> Cornea {
        let mut vertices: Vec<VertexNormal> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        let border: f32 = TAU / 12.0;
        let oz: f32 = border.cos();
        let oxy: f32 = border.sin();
        let cornea0 = f32xyz { x: 0.0,y: 0.0,z: 0.409, };
        let top = f32xyz { x: 0.0,y: 0.0,z: 1.087, };
        for k in 0..32 {
            let theta: f32 =((k as f32) * TAU) / 32.0;
            let vo = f32xyz { x: oxy * theta.cos(),y: oxy * theta.sin(),z: oz, };
            let no = vo - cornea0;
            let r = no.abs();
            let no = no / r;
            let nm = (((vo + top) / 2.0) - cornea0).norm();
            let vm = cornea0 + nm * r;
            vertices.push(VertexNormal { v: vo,n: no, });
            vertices.push(VertexNormal { v: vm,n: nm, });
        }
        vertices.push(VertexNormal { v: top,n: f32xyz { x: 0.0,y: 0.0,z: 1.0, }});
        for k in 0..31 {
            indices.push(k * 2);
            indices.push(k * 2 + 2);
            indices.push(k * 2 + 1);
            indices.push(k * 2 + 1);
            indices.push(k * 2 + 2);
            indices.push(k * 2 + 3);
            indices.push(k * 2 + 1);
            indices.push(k * 2 + 3);
            indices.push(64);
        }
        indices.push(62);
        indices.push(0);
        indices.push(63);
        indices.push(63);
        indices.push(0);
        indices.push(1);
        indices.push(63);
        indices.push(1);
        indices.push(64);
        let vs = VertexShader::new(r#"
            #version 420 core

            uniform mat4 u_projection;
            uniform mat4 u_modelview;
            uniform mat3 u_normal;

            layout(location = 0) in vec3 i_pos;
            layout(location = 1) in vec3 i_normal;

            out vec3 v_pos;
            out vec3 v_normal;

            void main() {
                v_pos = (u_modelview * vec4(i_pos,1.0)).xyz;
                v_normal = u_normal * i_normal;
                gl_Position = u_projection * vec4(v_pos,1.0);
            }
        "#).expect("Unable to create vertex shader.");
        let fs = FragmentShader::new(r#"
            #version 420 core

            uniform vec4 u_light_color;
            uniform vec3 u_light_dir;

            in vec3 v_pos;
            in vec3 v_normal;

            out vec4 o_frag;

            void main(void)
            {
                vec3 n = normalize(v_normal);
                vec3 pu = -v_pos;
                vec3 pun = normalize(pu);
                vec3 rn = reflect(-u_light_dir,n);
                float s = max(pow(dot(rn,pun),64.0),0.0);
                vec3 res = s * u_light_color.xyz;
                o_frag = vec4(res,1.0);
            }
        "#).expect("Unable to create fragment shader.");
        Cornea {
            cornea: VertexIndexBuffer::from(vertices,indices),
            shader: ShaderProgram::new(&vs,None,&fs).expect("Unable to create eye shader program."),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,light_dir: f32xyz,light_color: f32rgb) {
        let normal_matrix = f32m3x3::normal_from(modelview_matrix);
        unsafe {
            self.shader.bind();
            self.shader.set_uniform("u_projection",projection_matrix);
            self.shader.set_uniform("u_modelview",modelview_matrix);
            self.shader.set_uniform("u_normal",normal_matrix);
            self.shader.set_uniform("u_light_color",light_color);
            self.shader.set_uniform("u_light_dir",light_dir);
            self.cornea.bind();
            gl::DrawElements(gl::TRIANGLES,self.cornea.indices() as i32,gl::UNSIGNED_SHORT as u32,0 as *const ffi::c_void);
        }        
    }
}

pub struct Eye {
    sclera: Sclera,
    iris: Iris,
    pupil: Pupil,
    cornea: Cornea,
}

impl Eye {
    pub fn new() -> Eye {
        Eye {
            sclera: Sclera::new(),
            iris: Iris::new(),
            pupil: Pupil::new(),
            cornea: Cornea::new(),
        }
    }

    pub fn render_full(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4,light_dir: f32xyz,light_color: f32rgb,ambient_color: f32rgb,sclera_color: f32rgb,iris_color: f32rgb,) {
        unsafe {
            self.sclera.render_full(projection_matrix,modelview_matrix,light_dir,light_color,ambient_color,sclera_color);
            self.iris.render_full(projection_matrix,modelview_matrix,light_dir,light_color,ambient_color,iris_color);
            self.pupil.render_full(projection_matrix,modelview_matrix);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE,gl::ONE);
            self.cornea.render_full(projection_matrix,modelview_matrix,light_dir,light_color);
            gl::Disable(gl::BLEND);
        }
    }

    pub fn render_spec(&self,projection_matrix: f32m4x4,modelview_matrix: f32m4x4) {
        self.sclera.render_spec(projection_matrix,modelview_matrix);
        self.iris.render_spec(projection_matrix,modelview_matrix);
        self.pupil.render_spec(projection_matrix,modelview_matrix);
    }
}
