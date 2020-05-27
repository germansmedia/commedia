// (C) Copyright 2020, by Germans Media Technology & Services
// Commedia
// OpenGL 4.5 2D Framebuffer

use crate::math::*;
use std::marker;
use std::os::raw::c_void;
use gl::types::{GLint,GLuint};
use crate::*;

pub struct Framebuffer<T> {
    pub size: usizexy,
    tex: GLuint,
    rbo: GLuint,
    fbo: GLuint,
    _phantom: marker::PhantomData<T>,
}

impl<T: Pixel + GlPixelParams> Framebuffer<T> {
    pub fn new(size: usizexy) -> std::result::Result<Framebuffer<T>,()> {
        unsafe {
            let mut tex: GLuint = 0;
            let mut rbo: GLuint = 0;
            let mut fbo: GLuint = 0;
            gl::GenFramebuffers(1,&mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER,fbo);
            gl::GenTextures(1,&mut tex);
            gl::BindTexture(gl::TEXTURE_2D,tex);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_S,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_WRAP_T,gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MIN_FILTER,gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D,gl::TEXTURE_MAG_FILTER,gl::LINEAR as i32);
            gl::TexImage2D(gl::TEXTURE_2D,0,<T>::gl_internal_format() as GLint,size.x as i32,size.y as i32,0,<T>::gl_format(),<T>::gl_type(),0 as *const c_void);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER,gl::COLOR_ATTACHMENT0,gl::TEXTURE_2D,tex,0);
            gl::GenRenderbuffers(1,&mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER,rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER,gl::DEPTH_COMPONENT16,size.x as i32,size.y as i32);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER,gl::DEPTH_ATTACHMENT,gl::RENDERBUFFER,rbo);
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return Err(());
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER,0);
            return Ok(Framebuffer {
                size: size,
                tex: tex,
                rbo: rbo,
                fbo: fbo,
                _phantom: marker::PhantomData,
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER,self.fbo);
            gl::Viewport(0,0,self.size.x as i32,self.size.y as i32);
            gl::Scissor(0,0,self.size.x as i32,self.size.y as i32);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER,0);
        }
    }

    pub fn grab(&self) -> Image<T> {
        unsafe {
            let mut image = Image::new(self.size.clone());
            gl::GetTexImage(gl::TEXTURE_2D,0,<T>::gl_format(),<T>::gl_type(),image.data.as_ptr() as *mut c_void);
            image
        }
    }

    pub fn set(&self,image: &Image<T>) {
        unsafe {
            gl::TexImage2D(gl::TEXTURE_2D,0,<T>::gl_internal_format() as GLint,image.size.x as GLint,image.size.y as GLint,0,<T>::gl_format(),<T>::gl_type(),image.data.as_ptr() as *const c_void);
        }
    }
}

impl<T> Drop for Framebuffer<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1,&self.fbo);
            gl::DeleteRenderbuffers(1,&self.rbo);
            gl::DeleteTextures(1,&self.tex);
        }
    }
}