// Social Robotics Platform 03
// Desmond Germans, Ph.D
// OpenGL 4.5 2D vertex buffer

use crate::math::*;
use std::{marker,mem};
use std::os::raw::c_void;
use gl::types::{GLuint,GLint};

pub trait Vertex {
    fn prepare_vbo(vbo: GLuint);
}

pub trait Index {
}

impl Vertex for f32xyz {
    fn prepare_vbo(_vbo: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0,3,gl::FLOAT,gl::FALSE,mem::size_of::<Self>() as GLint,0 as *const c_void);
        }
    }
}

pub struct VertexNormal {
    pub v: f32xyz,
    pub n: f32xyz,
}

impl Vertex for VertexNormal {
    fn prepare_vbo(_vbo: GLuint) {
        unsafe {
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0,3,gl::FLOAT,gl::FALSE,mem::size_of::<Self>() as GLint,0 as *const c_void);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1,3,gl::FLOAT,gl::FALSE,mem::size_of::<Self>() as GLint,mem::size_of::<f32xyz>() as *const c_void);
        }
    }
}

impl Index for u16 { }

impl Index for u32 { }

pub struct VertexBuffer<V> {
    vao: GLuint,
    vbo: GLuint,
    vertices: usize,
    phantom_v: marker::PhantomData<V>,
}

impl<V: Vertex> VertexBuffer<V> {
    fn create() -> (GLuint,GLuint) {
        unsafe {
            let mut vao: GLuint = 0;
            gl::GenVertexArrays(1,&mut vao);
            gl::BindVertexArray(vao);
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1,&mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
            <V>::prepare_vbo(vbo);
            (vao,vbo)
        }
    }

    pub fn new(size: usize) -> VertexBuffer<V> {
        let (vao,vbo) = Self::create();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,(size * mem::size_of::<V>()) as isize,0 as *const c_void,gl::DYNAMIC_DRAW);
        }
        VertexBuffer {
            vao: vao,
            vbo: vbo,
            vertices: size,
            phantom_v: marker::PhantomData,
        }
    }

    pub fn from(array: Vec<V>) -> VertexBuffer<V> {
        let (vao,vbo) = Self::create();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,(array.len() * mem::size_of::<V>()) as isize,array.as_ptr() as *const c_void,gl::DYNAMIC_DRAW);
        }
        VertexBuffer {
            vao: vao,
            vbo: vbo,
            vertices: array.len(),
            phantom_v: marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn vertices(&self) -> usize {
        self.vertices
    }
}

impl<V> Drop for VertexBuffer<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1,&self.vbo);
            gl::DeleteVertexArrays(1,&self.vao);
        }
    }
}

pub struct VertexIndexBuffer<V,I> {
    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    vertices: usize,
    indices: usize,
    phantom_v: marker::PhantomData<V>,
    phantom_i: marker::PhantomData<I>,
}

impl<V: Vertex,I: Index> VertexIndexBuffer<V,I> {
    fn create() -> (GLuint,GLuint,GLuint) {
        unsafe {
            let mut vao: GLuint = 0;
            gl::GenVertexArrays(1,&mut vao);
            gl::BindVertexArray(vao);
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1,&mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER,vbo);
            let mut ebo: GLuint = 0;
            gl::GenBuffers(1,&mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER,ebo);
            <V>::prepare_vbo(vbo);
            (vao,vbo,ebo)
        }
    }

    pub fn new(vsize: usize,nsize: usize) -> VertexIndexBuffer<V,I> {
        let (vao,vbo,ebo) = Self::create();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,(vsize * mem::size_of::<V>()) as isize,0 as *const c_void,gl::DYNAMIC_DRAW);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,(nsize * mem::size_of::<I>()) as isize,0 as *const c_void,gl::DYNAMIC_DRAW);
        }
        VertexIndexBuffer {
            vao: vao,
            vbo: vbo,
            ebo: ebo,
            vertices: vsize,
            indices: nsize,
            phantom_v: marker::PhantomData,
            phantom_i: marker::PhantomData,
        }
    }

    pub fn from(vertices: Vec<V>,indices: Vec<I>) -> VertexIndexBuffer<V,I> {
        let (vao,vbo,ebo) = Self::create();
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER,(vertices.len() * mem::size_of::<V>()) as isize,vertices.as_ptr() as *const c_void,gl::DYNAMIC_DRAW);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,(indices.len() * mem::size_of::<I>()) as isize,indices.as_ptr() as *const c_void,gl::DYNAMIC_DRAW);
        }
        VertexIndexBuffer {
            vao: vao,
            vbo: vbo,
            ebo: ebo,
            vertices: vertices.len(),
            indices: indices.len(),
            phantom_v: marker::PhantomData,
            phantom_i: marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }

    pub fn vertices(&self) -> usize {
        self.vertices
    }

    pub fn indices(&self) -> usize {
        self.indices
    }
}

impl<V,I> Drop for VertexIndexBuffer<V,I> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1,&self.vbo);
            gl::DeleteBuffers(1,&self.ebo);
            gl::DeleteVertexArrays(1,&self.vao);
        }
    }
}
