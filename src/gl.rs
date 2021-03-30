use libc::{c_char, c_float, c_int, c_uint};

use core::slice;
use std::{convert::TryInto, ffi::CStr, fs::File, io::Write, ptr::null};

use crate::{FALSE_GL_VERSION, NATIVE_GL, PROGRAMS, Program, REPLACED_PROGRAMS, SHADERS, gl_def::{GL_FRAGMENT_SHADER, GL_PROGRAM_SEPARABLE, GL_TRUE, GL_VERSION, GL_VERTEX_SHADER}};

#[no_mangle]
pub extern "C" fn glGetString(flag: c_uint) -> *const c_char {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();

        let mut false_string: *const c_char = null();

        if flag == GL_VERSION {
            FALSE_GL_VERSION.with(|version_cell| {
                let version = version_cell.borrow_mut();

                false_string = version.as_ptr() as *const c_char;
            });
        } else {
            false_string = gl.get_string.unwrap()(flag);
        }

        return false_string;
    })
}

#[no_mangle]
pub extern "C" fn glClear(flag: c_uint) {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();

        // println!("glClear({})", flag);

        gl.clear.unwrap()(flag);
    });
}

#[no_mangle]
pub extern "C" fn glClearColor(r: c_float, g: c_float, b: c_float, a: c_float) {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();

        // println!("glClearColor({}, {}, {}, {})", r, g, b, a);

        gl.clear_color.unwrap()(r, g, b, a);
    });
}

#[no_mangle]
pub extern "C" fn glCreateShader(shader_type: c_int) -> c_int {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();

        let new_shader = gl.create_shader.unwrap()(shader_type);

        SHADERS.with(|shaders_cell| {
            let mut shaders = shaders_cell.borrow_mut();

            if shader_type == GL_VERTEX_SHADER {
                println!("Registered {} as 'GL_VERTEX_SHADER'", new_shader);
            } else if shader_type == GL_FRAGMENT_SHADER {
                println!("Registered {} as 'GL_FRAGMENT_SHADER'", new_shader);
            }

            shaders.insert(new_shader, shader_type);
        });

        return new_shader;
    })
}

#[no_mangle]
pub extern "C" fn glShaderSource(
    shader: c_int,
    size: isize,
    string: *const *const c_char,
    length: *const c_int,
) {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();

        println!("Oh damn a shader !!!");
        for i in 0..size {
            let source = unsafe { string.offset(i).read() };
            let length = if length.is_null() {
                0
            } else {
                unsafe { length.offset(i).read() }
            };

            let source_str;
            // null-terminated?
            if length == 0 {
                source_str = unsafe { CStr::from_ptr(source).to_str().unwrap() };
            } else {
                let raw_str = unsafe {
                    slice::from_raw_parts(source as *const u8, length.try_into().unwrap())
                };
                source_str = unsafe {
                    CStr::from_bytes_with_nul_unchecked(raw_str)
                        .to_str()
                        .unwrap()
                };
            }

            let shader_path = format!(
                "/home/vincent/Projects/probriquegl/shaders/shader_n{}.glsl",
                shader
            );
            println!("Writing to '{}'", shader_path);
            let mut shader_file = File::create(shader_path).unwrap();
            shader_file.write_all(source_str.as_bytes()).unwrap();
        }

        gl.shader_source.unwrap()(shader, size, string, length);
    });
}

#[no_mangle]
pub extern "C" fn glAttachShader(program: c_int, shader: c_int) {
    // First register this pair program <-> shader
    PROGRAMS.with(|programs_cell| {
        let mut programs = programs_cell.borrow_mut();

        if !programs.contains_key(&program) {
            programs.insert(
                program,
                Program {
                    old_vertex_shader: 0,
                    vertex_shader: 0,
                    old_fragment_shader: 0,
                    fragment_shader: 0,
                },
            );
        }

        let program = programs.get_mut(&program).unwrap();
        let shader_type =
            SHADERS.with(|shaders_cell| shaders_cell.borrow().get(&shader).unwrap().clone());
        if shader_type == GL_VERTEX_SHADER {
            program.old_vertex_shader = shader;
            program.vertex_shader = shader;
        } else if shader_type == GL_FRAGMENT_SHADER {
            program.old_fragment_shader = shader;
            program.fragment_shader = shader;
        } else {
            panic!("Unknow shader type (not Gl_VERTEX_SHADER, not GL_FRAGMENT_SHADER).");
        }
    });

    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();
        gl.program_parameter.unwrap()(program, GL_PROGRAM_SEPARABLE, GL_TRUE);
        gl.attach_shader.unwrap()(program, shader);
    })
}

#[no_mangle]
pub extern "C" fn glDeleteShader(_: c_int) {
    // println!("LoL nope!");
}

#[no_mangle]
pub extern "C" fn glUseProgram(program: c_int) {
    NATIVE_GL.with(|gl_cell| {
        let gl = gl_cell.borrow();
        REPLACED_PROGRAMS.with(|programs_cell| {
            let programs = programs_cell.borrow();

            if let Some(prog) = programs.get(&program) {
                gl.use_program.unwrap()(*prog);

                /*println!(
                    "Ok lol the game wants to use {} but i said {}",
                    program, *prog
                );*/
            } else {
                gl.use_program.unwrap()(program);
            }
        })
    });
}