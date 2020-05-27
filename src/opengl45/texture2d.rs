// (C) Copyright 2020, by Germans Media Technology & Services
// Commedia
// OpenGL 4.5 2D Texture

use crate::math::*;
use std::marker;
use std::os::raw::c_void;
use gl::types::{GLuint,GLint,GLenum};
use crate::*;

pub struct Texture2D<T> {
    tex: GLuint,
    phantom: marker::PhantomData<T>,
}

pub trait GlPixelParams {
    fn gl_internal_format() -> GLuint;
    fn gl_format() -> GLenum;
    fn gl_type() -> GLenum;
}

impl GlPixelParams for ARGB8 {
    fn gl_internal_format() -> GLuint { gl::RGBA as GLuint }
    fn gl_format() -> GLenum { gl::BGRA }
    fn gl_type() -> GLenum { gl::UNSIGNED_INT_8_8_8_8_REV }
}

impl<T: Pixel + GlPixelParams> Texture2D<T> {
    fn create() -> GLuint {
        unsafe {
            let mut tex: GLuint = 0;
            gl::GenTextures(1,&mut tex);
            gl::BindTexture(gl::TEXTURE_2D,tex);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            tex
        }
    }

    pub fn new(size: usizexy) -> Texture2D<T> {
        let tex = Self::create();
        unsafe {
            gl::TexStorage2D(gl::TEXTURE_2D,1,<T>::gl_internal_format(),size.x as i32,size.y as i32);
        }
        Texture2D {
            tex: tex,
            phantom: marker::PhantomData,
        }
    }

    pub fn from_image<U: Pixel>(image: Image<U>) -> Texture2D<T> {
        let tex = Self::create();
        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D,0,<T>::gl_internal_format() as GLint,image.size.x as GLint,image.size.y as GLint,0,<T>::gl_format(),<T>::gl_type(),image.data.as_ptr() as *const c_void);
        }
        Texture2D {
            tex: tex,
            phantom: marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D,self.tex);
        }
    }
}

impl<T> Drop for Texture2D<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1,&self.tex);
        }
    }
}
