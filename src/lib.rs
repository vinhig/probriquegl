#![allow(non_snake_case)] // yeah we want a BIG GL, not a little gl

#[macro_use]
extern crate lazy_static;

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    ffi::{CStr, CString},
    fs::File,
    io::{self, stdin, stdout, Read, Write},
    process::exit,
    ptr::null_mut,
    sync::{Arc, Mutex, RwLock},
};

use libc::{c_char, c_float, c_int, c_uint, c_void, dlerror, dlopen, dlsym};
use rand::Rng;

pub mod gl;
pub mod gl_def;
pub mod glx;

pub struct Glx {
    ready: bool,
    handle: *mut c_void,

    pub get_fb_configs: Option<fn(*const c_void, c_int, *const c_int) -> *mut c_void>,
    pub get_fb_config_attrib:
        Option<fn(*const c_void, *const c_void, c_int, *const c_int) -> c_int>,
    pub get_client_string: Option<fn(*const c_void, c_int) -> *const c_char>,
    pub query_extension: Option<fn(*const c_void, *const c_int, *const c_int) -> bool>,
    pub query_version: Option<fn(*const c_void, *const c_int, *const c_int) -> bool>,
    pub destroy_context: Option<fn(*const c_void, *const c_void)>,
    pub make_current: Option<fn(*const c_void, *const c_void, *const c_void) -> bool>,
    pub swap_buffers: Option<fn(*const c_void, *const c_void)>,
    pub query_extensions_string: Option<fn(*const c_void, c_int) -> *const c_char>,
    pub create_new_context:
        Option<fn(*const c_void, *const c_void, c_int, *const c_void, bool) -> *const c_void>,
    pub create_window: Option<
        fn(*const c_void, *const c_void, *const c_void, attrib_list: *const c_int) -> *const c_void,
    >,
    pub destroy_window: Option<fn(*const c_void, *const c_void)>,
    pub get_proc_address: Option<fn(*const c_char) -> *const c_void>,
    pub get_proc_address_arb: Option<fn(*const c_char) -> *const c_void>,
    pub get_visual_from_fb_config: Option<fn(*const c_void, *const c_void) -> *const c_void>,

    pub create_context_attribs_arb: Option<
        fn(
            *const c_void,
            *const c_void,
            *const c_void,
            direct: bool,
            *const c_int,
        ) -> *const c_void,
    >,
}

pub struct Gl {
    pub handle: *mut c_void,
    pub ready: bool,

    pub get_string: Option<fn(c_uint) -> *const c_char>,
    pub clear: Option<fn(c_uint)>,
    pub clear_color: Option<fn(c_float, c_float, c_float, c_float)>,
    pub create_shader: Option<fn(c_int) -> c_int>,
    pub shader_source: Option<fn(c_int, isize, *const *const c_char, *const c_int)>,
    pub attach_shader: Option<fn(c_int, c_int)>,
    pub compile_shader: Option<fn(c_int)>,
    pub link_program: Option<fn(c_int)>,
    pub use_program: Option<fn(c_int)>,
    pub create_program: Option<fn() -> c_int>,
    pub delete_program: Option<fn(c_int)>,
    pub delete_shader: Option<fn(c_int)>,
    pub get_shader_iv: Option<fn(c_int, c_uint, *const c_int)>,
    pub get_shader_info_log: Option<fn(c_int, c_int, *const c_int, *const c_char)>,
    pub program_parameter: Option<fn(c_int, c_uint, c_int)>,
}

fn load_fn(handle: *mut c_void, function: &str) -> *const c_void {
    let symbol = CString::new(function).unwrap();
    let sym = unsafe { dlsym(handle, symbol.as_ptr()) };

    let error = unsafe { dlerror() };
    if !error.is_null() {
        unsafe {
            let str = CStr::from_ptr(error).to_str().unwrap();
            println!("Yo wtf... {}", str);
            panic!();
        }
    }
    if sym.is_null() {
        panic!("Yo wtf...");
    }
    sym
}

impl Glx {
    pub fn init(&mut self) {
        if !self.ready {
            unsafe {
                dlerror();
            }
            let lib_path = CString::new("/usr/lib/libGL.so.1").unwrap();
            let lib = unsafe { dlopen(lib_path.as_ptr(), libc::RTLD_LAZY) };
            {
                let error = unsafe { dlerror() };
                if !error.is_null() {
                    unsafe {
                        let str = CStr::from_ptr(error).to_str().unwrap();
                        println!("Yo wtf... {}", str);
                        panic!();
                    }
                }
            }
            self.handle = lib;

            self.get_fb_configs = unsafe { std::mem::transmute(load_fn(lib, "glXGetFBConfigs")) };
            self.get_fb_config_attrib =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXGetFBConfigAttrib"))) };
            self.get_client_string =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXGetClientString"))) };
            self.query_extension =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXQueryExtension"))) };
            self.query_version =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXQueryVersion"))) };
            self.destroy_context =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXDestroyContext"))) };
            self.make_current =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXMakeCurrent"))) };
            self.swap_buffers =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXSwapBuffers"))) };
            self.query_extensions_string = unsafe {
                Some(std::mem::transmute(load_fn(
                    lib,
                    "glXQueryExtensionsString",
                )))
            };
            self.create_new_context =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXCreateNewContext"))) };
            self.create_window =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXCreateWindow"))) };
            self.destroy_window =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXDestroyWindow"))) };
            self.get_proc_address =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXGetProcAddress"))) };
            self.get_proc_address_arb =
                unsafe { Some(std::mem::transmute(load_fn(lib, "glXGetProcAddressARB"))) };
            self.get_visual_from_fb_config = unsafe {
                Some(std::mem::transmute(load_fn(
                    lib,
                    "glXGetVisualFromFBConfig",
                )))
            };
            self.create_context_attribs_arb = unsafe {
                Some(std::mem::transmute(load_fn(
                    lib,
                    "glXCreateContextAttribsARB",
                )))
            };

            NATIVE_GL.with(|gl_cell| {
                let mut gl = gl_cell.borrow_mut();
                gl.init(lib);
            });

            self.ready = true;

            // Launch a console in a separated thread
            std::thread::spawn(|| {
                let quit = false;
                while !quit {
                    // Ask for user command
                    let mut cmd = String::new();
                    print!(">>> ");
                    let stdin = stdin();
                    let _ = stdout().flush();
                    stdin.read_line(&mut cmd).unwrap();
                    if cmd == "quit\n" {
                        exit(-1);
                    }
                    println!("Executing {}", cmd);

                    // OpenGL operation should be done on the OpenGL thread
                    // So register a lamda to call at the end of the current frame
                    {
                        let mut m = CALLS.write().unwrap();

                        if cmd == "randcolor\n" {
                            m.push(Action::ClearColorRandomize);
                        } else if cmd.starts_with("reload") {
                            let number = cmd
                                .strip_prefix("reload ")
                                .unwrap()
                                .strip_suffix("\n")
                                .unwrap();
                            m.push(Action::ShaderReloading(number.parse::<c_int>().unwrap()));
                        }
                    }
                }
            });
        }
    }
}

impl Gl {
    pub fn init(&mut self, handle: *mut c_void) {
        if !self.ready {
            self.handle = handle;
            self.get_string = unsafe { std::mem::transmute(load_fn(handle, "glGetString")) };
            self.clear_color = unsafe { std::mem::transmute(load_fn(handle, "glClearColor")) };
            self.clear = unsafe { std::mem::transmute(load_fn(handle, "glClear")) };
            self.create_shader = unsafe { std::mem::transmute(load_fn(handle, "glCreateShader")) };
            self.shader_source = unsafe { std::mem::transmute(load_fn(handle, "glShaderSource")) };
            self.attach_shader = unsafe { std::mem::transmute(load_fn(handle, "glAttachShader")) };
            self.compile_shader =
                unsafe { std::mem::transmute(load_fn(handle, "glCompileShader")) };
            self.link_program = unsafe { std::mem::transmute(load_fn(handle, "glLinkProgram")) };
            self.use_program = unsafe { std::mem::transmute(load_fn(handle, "glUseProgram")) };
            self.create_program =
                unsafe { std::mem::transmute(load_fn(handle, "glCreateProgram")) };
            self.delete_program =
                unsafe { std::mem::transmute(load_fn(handle, "glDeleteProgram")) };
            self.delete_shader = unsafe { std::mem::transmute(load_fn(handle, "glDeleteShader")) };
            self.get_shader_iv = unsafe { std::mem::transmute(load_fn(handle, "glGetShaderiv")) };
            self.get_shader_info_log =
                unsafe { std::mem::transmute(load_fn(handle, "glGetShaderInfoLog")) };
            self.program_parameter =
                unsafe { std::mem::transmute(load_fn(handle, "glProgramParameteri")) };

            self.ready = true;
        }
    }
}

pub struct Program {
    old_vertex_shader: c_int,
    vertex_shader: c_int,
    old_fragment_shader: c_int,
    fragment_shader: c_int,
}

pub enum Action {
    ClearColorRandomize,
    ShaderReloading(c_int),
}

thread_local! {
    pub static NATIVE_GLX: RefCell<Glx> = RefCell::new(Glx {
        ready: false,
        handle: null_mut(),

        get_fb_configs: None,
        get_fb_config_attrib: None,
        get_client_string: None,
        query_extension: None,
        query_version: None,
        destroy_context: None,
        make_current: None,
        swap_buffers: None,
        query_extensions_string: None,
        create_new_context: None,
        create_window: None,
        destroy_window: None,
        get_proc_address: None,
        get_proc_address_arb: None,
        get_visual_from_fb_config: None,

        create_context_attribs_arb: None,
    });

    pub static NATIVE_GL: RefCell<Gl> = RefCell::new(Gl {
        handle: null_mut(),
        ready: false,
        get_string: None,
        clear: None,
        clear_color: None,
        create_shader: None,
        shader_source: None,
        attach_shader: None,
        compile_shader: None,
        link_program: None,
        use_program: None,
        create_program: None,
        delete_program: None,
        delete_shader: None,
        get_shader_iv: None,
        get_shader_info_log: None,
        program_parameter: None,
    });

    pub static FRONT_GL: RefCell<*mut c_void> = RefCell::new(null_mut());

    pub static FALSE_GL_VERSION: RefCell<String> = RefCell::new("4.2".to_string());

    pub static PROGRAMS: RefCell<HashMap<c_int, Program>> = RefCell::new(HashMap::new());

    pub static SHADERS: RefCell<HashMap<c_int, c_int>> = RefCell::new(HashMap::new());

    pub static REPLACED_PROGRAMS: RefCell<HashMap<c_int, c_int>> = RefCell::new(HashMap::new());

}

lazy_static! {
    pub static ref CALLS: RwLock<Vec<Action>> = RwLock::new(Vec::new());
}
