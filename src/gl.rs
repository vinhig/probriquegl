use libc::{c_char, c_float, c_uint};

use std::ptr::null;

use crate::{gl_def::GL_VERSION, FALSE_GL_VERSION, NATIVE_GL};

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
