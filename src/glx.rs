use std::ffi::CStr;

use libc::{c_char, c_int, c_void};

use crate::{FALSE_GL_VERSION, NATIVE_GLX};

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
            let proc_addr: *const c_void;
            if proc_str == "glXCreateContextAttribsARB" {
                proc_addr = glXCreateContextAttribsARB as *const c_void;
            } else if proc_str == "glGetString" {
                proc_addr = crate::gl::glGetString as *const c_void;
            } else {
                proc_addr = glx.get_proc_address.unwrap()(proc_name);
            }
            println!("get_proc_address({}) = {:?}", proc_str, proc_addr);
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
