// Social Robotics Platform 04
// Desmond Germans, Ph.D
// Context3D for Linux with XCB using glX, exposing OpenGL 4.5

use std::{ffi,os,ptr,fmt};
#[doc(no_inline)]
extern crate x11;
use x11::{glx,xlib::*};
#[doc(no_inline)]
extern crate gl;

const GLX_CONTEXT_MAJOR_VERSION_ARB: u32 = 0x2091;
const GLX_CONTEXT_MINOR_VERSION_ARB: u32 = 0x2092;

type GlXCreateContextAttribsARBProc = unsafe extern "C" fn(dpy: *mut Display,fbc: glx::GLXFBConfig,share_context: glx::GLXContext,direct: Bool,attribs: *const os::raw::c_int) -> glx::GLXContext;

fn load_function(name: &str) -> *mut os::raw::c_void {
    let newname = ffi::CString::new(name).unwrap();
    let pointer: *mut os::raw::c_void = unsafe { std::mem::transmute(glx::glXGetProcAddress(newname.as_ptr() as *const u8)) };
    if pointer.is_null() { panic!("Canvas: unable to access {}", name); }
    pointer
}

pub enum Context3DError {
    Connection,
    Driver,
}

impl fmt::Debug for Context3DError {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Context3DError::Connection => { write!(f,"XCB connection error.") },
            Context3DError::Driver => { write!(f,"GLX driver error.") },
        }
    }
}

pub struct Context3D {
    connection: xcb::Connection,
    //depth: i32,
    //visualid: u64,
    //colormap: u32,
    //rootwindow: u32,
    hidden_window: u32,
    context: glx::GLXContext,
}

impl Context3D {
    pub fn new() -> std::result::Result<Context3D,Context3DError> {
        let (connection,_screen_number) = if let Ok((c,n)) = xcb::Connection::connect_with_xlib_display() {
            (c,n)
        }
        else {
            return Err(Context3DError::Connection);
        };
        connection.set_event_queue_owner(xcb::EventQueueOwner::Xcb);
        let mut glxmaj: os::raw::c_int = 0;
        let mut glxmin: os::raw::c_int = 0;
        unsafe { if glx::glXQueryVersion(connection.get_raw_dpy(),&mut glxmaj as *mut os::raw::c_int,&mut glxmin as *mut os::raw::c_int) == 0 { return Err(Context3DError::Driver) } };
        if (glxmaj * 100 + glxmin) < 103 { return Err(Context3DError::Driver); }
        let attribs = [
            glx::GLX_X_RENDERABLE,  1,
            glx::GLX_DRAWABLE_TYPE, glx::GLX_WINDOW_BIT,
            glx::GLX_RENDER_TYPE,   glx::GLX_RGBA_BIT,
            glx::GLX_X_VISUAL_TYPE, glx::GLX_TRUE_COLOR,
            glx::GLX_RED_SIZE,      8,
            glx::GLX_GREEN_SIZE,    8,
            glx::GLX_BLUE_SIZE,     8,
            glx::GLX_ALPHA_SIZE,    8,
            glx::GLX_DEPTH_SIZE,    24,
            glx::GLX_STENCIL_SIZE,  8,
            glx::GLX_DOUBLEBUFFER,  1,
            0,
        ];
        let mut fbcount: os::raw::c_int = 0;
        let fbconfigs = unsafe { glx::glXChooseFBConfig(connection.get_raw_dpy(),0,attribs.as_ptr(),&mut fbcount as *mut os::raw::c_int) };
        if fbcount == 0 { return Err(Context3DError::Driver); }
        let fbconfig = unsafe { *fbconfigs };
        unsafe { XFree(fbconfigs as *mut os::raw::c_void) };
        let visual = unsafe { glx::glXGetVisualFromFBConfig(connection.get_raw_dpy(),fbconfig) };
        let screen = unsafe { (*visual).screen };
        let visual_screen = if let Some(screen) = connection.get_setup().roots().nth(screen as usize) {
            screen
        }
        else {
            return Err(Context3DError::Driver);
        };
        let depth = unsafe { (*visual).depth };
        let visualid = unsafe { (*visual).visualid };
        let extensions = if let Ok(e) = unsafe { ffi::CStr::from_ptr(glx::glXQueryExtensionsString(connection.get_raw_dpy(),screen)) }.to_str() {
            e
        }
        else {
            return Err(Context3DError::Driver);
        };
        let mut found = false;
        for extension in extensions.split(" ") {
            if extension == "GLX_ARB_create_context" {
                found = true;
                break;
            }
        }
        if !found {
            return Err(Context3DError::Driver);
        }
        let glx_create_context_attribs: GlXCreateContextAttribsARBProc = unsafe { std::mem::transmute(load_function("glXCreateContextAttribsARB")) };
        let rootwindow = visual_screen.root();
        let hidden_window = connection.generate_id();
        let colormap = connection.generate_id();
        xcb::create_colormap(&connection,xcb::COLORMAP_ALLOC_NONE as u8,colormap,rootwindow,visualid as u32);
        let values = [
            (xcb::CW_EVENT_MASK,0),
            (xcb::CW_COLORMAP,colormap),
        ];
        xcb::create_window(&connection,depth as u8,hidden_window,rootwindow,0,0,1,1,0,xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,visualid as u32,&values);
        connection.flush();
        unsafe { XSync(connection.get_raw_dpy(),False) };
        let context_attribs: [os::raw::c_int; 5] = [
            GLX_CONTEXT_MAJOR_VERSION_ARB as os::raw::c_int, 4,
            GLX_CONTEXT_MINOR_VERSION_ARB as os::raw::c_int, 5,
            0,
        ];
        let context = unsafe { glx_create_context_attribs(connection.get_raw_dpy(),fbconfig,ptr::null_mut(),True,&context_attribs[0] as *const os::raw::c_int) };
        connection.flush();
        unsafe { XSync(connection.get_raw_dpy(), False) };
        if context.is_null() {
            return Err(Context3DError::Driver);
        }
        if unsafe { glx::glXIsDirect(connection.get_raw_dpy(),context) } == 0 {
            return Err(Context3DError::Driver);
        }
        unsafe { glx::glXMakeCurrent(connection.get_raw_dpy(),hidden_window as XID,context) };
        gl::load_with(|symbol| load_function(&symbol));
        Ok(Context3D {
            connection: connection,
            //depth: depth,
            //visualid: visualid,
            //rootwindow: rootwindow,
            hidden_window: hidden_window,
            //colormap: colormap,
            context: context,
        })
    }
}

impl Drop for Context3D {
    fn drop(&mut self) {
        unsafe { glx::glXMakeCurrent(self.connection.get_raw_dpy(),0,ptr::null_mut()); }
        xcb::unmap_window(&self.connection,self.hidden_window);
        xcb::destroy_window(&self.connection,self.hidden_window);
        unsafe { glx::glXDestroyContext(self.connection.get_raw_dpy(),self.context); }
    }
}
