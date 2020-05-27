// Social Robotics Platform 03
// Desmond Germans, Ph.D
// OpenGL 4.5 shader

use crate::math::*;
use std::{ffi::{CStr,CString},ptr::{null,null_mut}};
#[doc(no_inline)]
extern crate gl;
use gl::types::{GLchar,GLint,GLuint,GLfloat};

pub struct VertexShader {
    vs: GLuint,
}

pub struct FragmentShader {
    fs: GLuint,
}

pub struct GeometryShader {
    gs: GLuint,
}

pub struct ShaderProgram {
    sp: GLuint,
}

fn create_shader(stype: GLuint,src: &str) -> std::result::Result<GLuint,String> {
    unsafe {
        let s = gl::CreateShader(stype);
        let vcstr = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(s,1,&vcstr.as_ptr(),null());
        gl::CompileShader(s);
        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(s,gl::COMPILE_STATUS,&mut success);
        let mut info_log: Vec<GLchar> = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderInfoLog(s,512,null_mut(),info_log.as_mut_ptr() as *mut GLchar);
        let c_str: &CStr = CStr::from_ptr(info_log.as_ptr());
        let str_slice: &str = c_str.to_str().unwrap();
        if str_slice.len() > 0 {
            let stype_name = match stype {
                gl::VERTEX_SHADER => "vertex shader",
                gl::FRAGMENT_SHADER => "fragment shader",
                gl::GEOMETRY_SHADER => "geometry shader",
                _ => "unknown shader",
            };
            println!("create_shader for {}: {}",stype_name,str_slice);
        }
        if success == gl::TRUE as GLint {
            Ok(s)
        }
        else {
            Err(str_slice.to_owned())
        }
    }
}

impl VertexShader {
    pub fn new(src: &str) -> std::result::Result<VertexShader,String> {
        match create_shader(gl::VERTEX_SHADER,src) {
            Ok(s) => {
                Ok(VertexShader {
                    vs: s,
                })
            },
            Err(s) => {
                Err(s)
            },
        }
    }
}

impl Drop for VertexShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.vs); }
    }
}

impl FragmentShader {
    pub fn new(src: &str) -> std::result::Result<FragmentShader,String> {
        match create_shader(gl::FRAGMENT_SHADER,src) {
            Ok(s) => {
                Ok(FragmentShader {
                    fs: s,
                })
            },
            Err(s) => {
                Err(s)
            },
        }
    }
}

impl Drop for FragmentShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.fs); }
    }
}

impl GeometryShader {
    pub fn new(src: &str) -> std::result::Result<GeometryShader,String> {
        match create_shader(gl::GEOMETRY_SHADER,src) {
            Ok(s) => {
                Ok(GeometryShader {
                    gs: s,
                })
            },
            Err(s) => {
                Err(s)
            },
        }
    }
}

impl Drop for GeometryShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.gs); }
    }
}

impl ShaderProgram {
    pub fn new(vs: &VertexShader,gs: Option<&GeometryShader>,fs: &FragmentShader) -> std::result::Result<ShaderProgram,String> {
        unsafe {
            let sp = gl::CreateProgram();
            gl::AttachShader(sp,vs.vs);
            if let Some(gs) = gs {
                gl::AttachShader(sp,gs.gs);
            }
            gl::AttachShader(sp,fs.fs);
            gl::LinkProgram(sp);
            let mut success = gl::FALSE as GLint;
            gl::GetProgramiv(sp,gl::LINK_STATUS,&mut success);
            let mut info_log: Vec<GLchar> = Vec::with_capacity(512);
            info_log.set_len(512 - 1);
            gl::GetProgramInfoLog(sp,512,null_mut(),info_log.as_mut_ptr() as *mut GLchar);
            let c_str: &CStr = CStr::from_ptr(info_log.as_ptr());
            let str_slice: &str = c_str.to_str().unwrap();
            if str_slice.len() > 0 {
                println!("ShaderProgram::new: {}",str_slice);
            }
            if success == gl::TRUE as GLint {
                Ok(ShaderProgram {
                    sp: sp,
                })
            }
            else {
                Err(str_slice.to_owned())
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.sp);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.sp); }
    }
}

pub trait SetUniform<T> {
    fn set_uniform(&self,name: &str,value: T);
}

impl SetUniform<f32m3x3> for ShaderProgram {
    fn set_uniform(&self,name: &str,value: f32m3x3) {
        let cname = CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix3fv(gl::GetUniformLocation(self.sp,cname.as_ptr() as *const GLchar),1,gl::FALSE,&value as *const f32m3x3 as *const GLfloat);
        }
    }
}

impl SetUniform<f32m4x4> for ShaderProgram {
    fn set_uniform(&self,name: &str,value: f32m4x4) {
        let cname = CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix4fv(gl::GetUniformLocation(self.sp,cname.as_ptr() as *const GLchar),1,gl::FALSE,&value as *const f32m4x4 as *const GLfloat);
        }
    }
}

impl SetUniform<f32xyz> for ShaderProgram {
    fn set_uniform(&self,name: &str,value: f32xyz) {
        let cname = CString::new(name).unwrap();
        unsafe {
            gl::Uniform3fv(gl::GetUniformLocation(self.sp,cname.as_ptr() as *const GLchar),1,&value as *const f32xyz as *const GLfloat);
        }
    }
}

impl SetUniform<f32xyzw> for ShaderProgram {
    fn set_uniform(&self,name: &str,value: f32xyzw) {
        let cname = CString::new(name).unwrap();
        unsafe {
            gl::Uniform4fv(gl::GetUniformLocation(self.sp,cname.as_ptr() as *const GLchar),1,&value as *const f32xyzw as *const GLfloat);
        }
    }
}

impl SetUniform<f32rgb> for ShaderProgram {
    fn set_uniform(&self,name: &str,value: f32rgb) {
        self.set_uniform(name,f32xyzw { x: value.r,y: value.g,z: value.b,w: 1.0, });
    }
}
