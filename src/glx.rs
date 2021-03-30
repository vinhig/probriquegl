use core::panic;
use std::{
    borrow::BorrowMut,
    convert::TryInto,
    ffi::{CStr, CString},
    fs::{self, File},
    io::Read,
    ptr::null,
};

use libc::{c_char, c_int, c_void, dlerror, dlsym};
use rand::Rng;

use crate::{
    gl_def::{
        GL_COMPILE_STATUS, GL_FALSE, GL_FRAGMENT_SHADER, GL_INFO_LOG_LENGTH, GL_VERTEX_SHADER,
    },
    CALLS, FALSE_GL_VERSION, FRONT_GL, NATIVE_GL, NATIVE_GLX, PROGRAMS, REPLACED_PROGRAMS, SHADERS,
};

#[no_mangle]
pub extern "C" fn glXGetFBConfigs(
    display: *const c_void,
    screen: c_int,
    nelements: *const c_int,
) -> *mut c_void {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.get_fb_configs.unwrap()(display, screen, nelements);
    })
}

#[no_mangle]
pub extern "C" fn glXGetFBConfigAttrib(
    display: *const c_void,
    config: *mut c_void,
    attribute: c_int,
    value: *const c_int,
) -> c_int {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.get_fb_config_attrib.unwrap()(display, config, attribute, value);
    })
}

#[no_mangle]
pub extern "C" fn glXGetClientString(display: *const c_void, name: c_int) -> *const c_char {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.get_client_string.unwrap()(display, name);
    })
}

#[no_mangle]
pub extern "C" fn glXQueryExtension(
    display: *const c_void,
    error_base: *const c_int,
    event_base: *const c_int,
) -> bool {
    NATIVE_GLX.with(|glx_cell| {
        let mut glx = glx_cell.borrow_mut();
        glx.init();

        return glx.query_extension.unwrap()(display, error_base, event_base);
    })
}

#[no_mangle]
pub extern "C" fn glXQueryVersion(
    display: *const c_void,
    major: *const c_int,
    minor: *const c_int,
) -> bool {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        // println!("glXQueryVersion({:?}, {:?}, {:?})", display, major, minor);

        let cool = glx.query_version.unwrap()(display, major, minor);

        return cool;
    })
}

#[no_mangle]
pub extern "C" fn glXDestroyContext(display: *const c_void, context: *const c_void) {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.destroy_context.unwrap()(display, context);
    })
}

#[no_mangle]
pub extern "C" fn glXMakeCurrent(
    display: *const c_void,
    drawable: *const c_void,
    context: *const c_void,
) -> bool {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.make_current.unwrap()(display, drawable, context);
    })
}

#[no_mangle]
pub extern "C" fn glXSwapBuffers(display: *const c_void, drawable: *const c_void) {
    // Issue all previous job
    let mut calls = CALLS.write().unwrap();

    for action in calls.iter() {
        match action {
            crate::Action::ClearColorRandomize => {
                let mut rng = rand::thread_rng();
                NATIVE_GL.with(|cell| {
                    let gl = cell.borrow();
                    gl.clear_color.unwrap()(
                        rng.gen::<f32>(),
                        rng.gen::<f32>(),
                        rng.gen::<f32>(),
                        1.0,
                    );
                })
            }
            crate::Action::ShaderReloading(old_shader) => {
                // Read file
                let shader_path = format!(
                    "/home/vincent/Projects/probriquegl/shaders/shader_n{}.glsl",
                    old_shader
                );
                let shader_source = fs::read_to_string(shader_path).unwrap() + "\0";

                let ugly_source: *const *const i8 = &(shader_source.as_ptr() as *const i8);

                NATIVE_GL.with(|gl_cell| {
                    let gl = gl_cell.borrow();

                    // Get type of shader to reload
                    let shader_type = SHADERS.with(|shaders_cell| {
                        let shaders = shaders_cell.borrow_mut();
                        (*shaders.get(&old_shader).unwrap()).clone()
                    });

                    // Recreate a new shader
                    let new_shader = gl.create_shader.unwrap()(shader_type);

                    // Compile it!!!
                    gl.shader_source.unwrap()(new_shader, 1, ugly_source, null());
                    gl.compile_shader.unwrap()(new_shader);

                    // Check it
                    let mut result = GL_FALSE;
                    gl.get_shader_iv.unwrap()(
                        new_shader,
                        GL_COMPILE_STATUS,
                        &mut result as *mut c_int,
                    );

                    let mut length = GL_FALSE;
                    gl.get_shader_iv.unwrap()(
                        new_shader,
                        GL_INFO_LOG_LENGTH,
                        &mut length as *mut c_int,
                    );
                    length += 1;

                    if result == GL_FALSE {
                        let mut log: Vec<u8> = Vec::new();
                        log.resize(length.try_into().unwrap(), 0);
                        gl.get_shader_info_log.unwrap()(
                            new_shader,
                            length + 1,
                            null(),
                            log.as_ptr() as *const i8,
                        );
                        let log_str = String::from_utf8(log).unwrap();
                        println!("{}", log_str);
                        panic!("Unable to compile shader, stopping here.");
                    }

                    PROGRAMS.with(|programs_cell| {
                        let mut program: i32 = 0;
                        let mut programs = programs_cell.borrow_mut();

                        let mut vertex_shader = 0;
                        let mut fragment_shader = 0;

                        if shader_type == GL_VERTEX_SHADER {
                            for (p, detail) in programs.iter_mut() {
                                if detail.old_vertex_shader == *old_shader {
                                    program = (*p).clone();
                                    
                                    vertex_shader = new_shader;
                                    fragment_shader = detail.fragment_shader;

                                    gl.delete_shader.unwrap()(detail.vertex_shader);

                                    detail.vertex_shader = new_shader;
                                    break;
                                }
                            }
                        } else if shader_type == GL_FRAGMENT_SHADER {
                            for (p, detail) in programs.iter_mut() {
                                if detail.old_fragment_shader == *old_shader {
                                    program = (*p).clone();

                                    vertex_shader = detail.vertex_shader;
                                    fragment_shader = new_shader;

                                    gl.delete_shader.unwrap()(detail.fragment_shader);

                                    detail.fragment_shader = new_shader;

                                    println!("Here my boy");
                                    break;
                                }
                            }
                        } else {
                            panic!("Unknown GL_WTF_SHADER");
                        }

                        if program != 0 {
                            let old_program = program;
                            let new_program = gl.create_program.unwrap()();

                            gl.attach_shader.unwrap()(new_program, vertex_shader);
                            gl.attach_shader.unwrap()(new_program, fragment_shader);
                            gl.link_program.unwrap()(new_program);

                            // Delete previous program
                            REPLACED_PROGRAMS.with(|cell| {
                                if let Some(previous_program) =
                                    cell.borrow_mut().insert(old_program, new_program)
                                {
                                    // gl.delete_program.unwrap()(previous_program);
                                } else {
                                    // gl.delete_program.unwrap()(program);
                                }
                            });

                            println!(
                                "Create new program {} that will replace {}",
                                new_program, old_program
                            );
                        } else {
                            panic!("GL_WTF_PROGRAM");
                        }
                    });
                });
            }
        }
    }

    calls.clear();

    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.swap_buffers.unwrap()(display, drawable);
    })
}

#[no_mangle]
pub extern "C" fn glXQueryExtensionsString(display: *const c_void, screen: c_int) -> *const c_char {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        let extensions = glx.query_extensions_string.unwrap()(display, screen);

        /*unsafe {
            let str = CStr::from_ptr(extensions);
            println!("Extensions: {}", str.to_str().unwrap());
        }*/

        return extensions;
    })
}

#[no_mangle]
pub extern "C" fn glXCreateNewContext(
    display: *const c_void,
    config: *const c_void,
    render_type: c_int,
    share_list: *const c_void,
    direct: bool,
) -> *const c_void {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.create_new_context.unwrap()(display, config, render_type, share_list, direct);
    })
}

#[no_mangle]
pub extern "C" fn glXCreateWindow(
    display: *const c_void,
    config: *const c_void,
    window: *const c_void,
    attrib_list: *const c_int,
) -> *const c_void {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.create_window.unwrap()(display, config, window, attrib_list);
    })
}

#[no_mangle]
pub extern "C" fn glXDestroyWindow(display: *const c_void, window: *const c_void) {
    NATIVE_GLX.with(|glx_cell| {
        let glx = glx_cell.borrow();

        return glx.destroy_window.unwrap()(display, window);
    })
}

fn get_proc_address(proc_name: *const i8) -> *const c_void {
    NATIVE_GLX.with(|glx_cell| {
        let mut glx = glx_cell.borrow_mut();
        glx.init();

        unsafe {
            let proc_str = CStr::from_ptr(proc_name).to_str().unwrap();
            let mut proc_addr: *const c_void = null();
            if proc_str == "glXCreateContextAttribsARB" {
                proc_addr = glXCreateContextAttribsARB as *const c_void;
            } else if proc_str == "glGetString" {
                proc_addr = crate::gl::glGetString as *const c_void;
            } else {
                // Check if we have it before asking glx
                FRONT_GL.with(|lib_cell| {
                    let lib = lib_cell.borrow();
                    proc_addr = dlsym(*lib, proc_name);
                    let error = dlerror();
                    if !error.is_null() {
                        if !proc_str.starts_with("glX") {
                            /*println!(
                                "TODO: Hook up `{}`.\n\t{}",
                                proc_str,
                                CStr::from_ptr(error).to_str().unwrap()
                            );*/
                        }
                        // Sad, we don't have it yet, let's pass on
                        proc_addr = glx.get_proc_address.unwrap()(proc_name);
                    }
                });
            }
            // println!("get_proc_address({}) = {:?}", proc_str, proc_addr);
            return proc_addr;
        }
    })
}

#[no_mangle]
pub extern "C" fn glXGetProcAddress(proc_name: *const c_char) -> *const c_void {
    get_proc_address(proc_name)
}

#[no_mangle]
pub extern "C" fn glXGetProcAddressARB(proc_name: *const c_char) -> *const c_void {
    get_proc_address(proc_name)
}

#[no_mangle]
pub extern "C" fn glXGetVisualFromFBConfig(
    display: *const c_void,
    config: *const c_void,
) -> *const c_void {
    NATIVE_GLX.with(|glx_cell| {
        let mut glx = glx_cell.borrow_mut();
        glx.init();

        return glx.get_visual_from_fb_config.unwrap()(display, config);
    })
}

#[no_mangle]
pub extern "C" fn ask_me_for_42() {
    println!("Take this with u: 42");
}

#[no_mangle]
pub extern "C" fn glXCreateContextAttribsARB(
    display: *const c_void,
    config: *const c_void,
    context: *const c_void,
    direct: bool,
    attrib_list: *mut c_int,
) -> *const c_void {
    // we lie here about the abilities of our system
    let mut original_major = 0;
    let mut original_minor = 0;
    let mut i = 0;
    while original_major == 0 || original_minor == 0 {
        unsafe {
            let key = attrib_list.offset(i).read();
            let value = attrib_list.offset(i + 1).read();

            if key == 0x2092 {
                original_minor = value;
                attrib_list.offset(i + 1).write(3);
            } else if key == 0x2091 {
                original_major = value;
                attrib_list.offset(i + 1).write(3);
            } else if key == 0 {
                break;
            }
        }

        i += 2;
    }

    println!(
        "The Game® wanted to set OpenGL version to {}.{}",
        original_major, original_minor
    );
    println!("But I saied no.");

    FALSE_GL_VERSION.with(|version_cell| {
        let mut version = version_cell.borrow_mut();
        *version = format!("{}.{} ProbriqueGL 0.0.1", original_major, original_minor);

        println!("From now on, we're officially using '{}'.", version);
    });

    NATIVE_GLX.with(|glx_cell| {
        let mut glx = glx_cell.borrow_mut();
        glx.init();

        return glx.create_context_attribs_arb.unwrap()(
            display,
            config,
            context,
            direct,
            attrib_list,
        );
    })
}
