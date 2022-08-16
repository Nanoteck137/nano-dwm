use x11::xlib::{
    Display, XSetErrorHandler, XErrorEvent, XSelectInput, XDefaultRootWindow,
    SubstructureRedirectMask, XSync, BadWindow, BadDrawable, BadMatch,
    BadAccess, Window,
};
use std::ffi::{c_int, c_uint, c_uchar, c_char, c_float, CString};

const X_CONFIGURE_WINDOW: c_uchar = 12;
const X_GRAB_BUTTON: c_uchar = 28;
const X_GRAB_KEY: c_uchar = 33;
const X_SET_INPUT_FOCUS: c_uchar = 42;
const X_COPY_AREA: c_uchar = 62;
const X_POLY_SEGMENT: c_uchar = 66;
const X_POLY_TEXT_8: c_uchar = 74;
const X_POLY_FILL_RECTANGLE: c_uchar = 70;

static mut DEFAULT_ERROR_HANDLER: Option<
    unsafe extern "C" fn(*mut Display, *mut XErrorEvent) -> c_int,
> = None;

extern "C" {
    fn resize(
        client: *mut Client,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        interact: i32,
    );
}

#[repr(C)]
pub struct Client {
    name: [c_char; 256],
    min_aspect: c_float,
    max_aspect: c_float,

    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,

    old_x: c_int,
    old_y: c_int,
    old_width: c_int,
    old_height: c_int,

    base_width: c_int,
    base_height: c_int,
    inc_width: c_int,
    inc_height: c_int,
    max_width: c_int,
    max_height: c_int,
    min_width: c_int,
    min_height: c_int,

    border_width: c_int,
    old_border_width: c_int,

    tags: c_uint,

    is_fixed: c_int,
    is_floating: c_int,
    is_urgent: c_int,
    never_focus: c_int,
    old_state: c_int,
    is_fullscreen: c_int,

    next: *mut Client,
    stack_next: *mut Client,
    monitor: *mut Monitor,

    window: Window,
}

impl Client {
    fn is_visable(&self) -> bool {
        let monitor = unsafe { &*self.monitor };
        self.tags & monitor.tagset[monitor.seltags as usize] > 0
    }
}

// typedef struct {
// 	const char *symbol;
// 	void (*arrange)(Monitor *);
// } Layout;

#[repr(C)]
pub struct Layout {
    symbol: *const c_char,
    arrange: unsafe extern "C" fn(*mut Monitor),
}

#[derive(Debug)]
#[repr(C)]
pub struct Monitor {
    ltsymbol: [c_uchar; 16],
    mfact: c_float,
    nmaster: c_int,
    num: c_int,
    by: c_int,

    mx: c_int,
    my: c_int,
    mw: c_int,
    mh: c_int,

    wx: c_int,
    wy: c_int,
    ww: c_int,
    wh: c_int,

    seltags: c_uint,
    sellt: c_uint,
    tagset: [c_uint; 2],

    show_bar: c_int,
    top_bar: c_int,

    clients: *mut Client,
    sel: *mut Client,
    stack: *mut Client,

    next: *mut Monitor,
    bar_window: Window,
    lt: [*mut Layout; 2],
}

#[no_mangle]
pub unsafe extern "C" fn hello_world_rust() {
    println!("Hello World from Rust");
}

#[no_mangle]
pub unsafe extern "C" fn print_monitor(monitor: *mut Monitor) {
    println!("Monitor: {:#?}", &*monitor);
}

unsafe extern "C" fn x_error_start(
    _display: *mut Display,
    _error_event: *mut XErrorEvent,
) -> c_int {
    panic!("Another Window Manager is running");
}

unsafe extern "C" fn x_error(
    display: *mut Display,
    error_event: *mut XErrorEvent,
) -> c_int {
    let ee = &*error_event;

    if ee.error_code == BadWindow ||
        (ee.request_code == X_SET_INPUT_FOCUS && ee.error_code == BadMatch) ||
        (ee.request_code == X_POLY_TEXT_8 && ee.error_code == BadDrawable) ||
        (ee.request_code == X_POLY_FILL_RECTANGLE &&
            ee.error_code == BadDrawable) ||
        (ee.request_code == X_POLY_SEGMENT && ee.error_code == BadDrawable) ||
        (ee.request_code == X_CONFIGURE_WINDOW && ee.error_code == BadMatch) ||
        (ee.request_code == X_GRAB_BUTTON && ee.error_code == BadAccess) ||
        (ee.request_code == X_GRAB_KEY && ee.error_code == BadAccess) ||
        (ee.request_code == X_COPY_AREA && ee.error_code == BadDrawable)
    {
        return 0;
    }

    println!("X Error");
    if let Some(f) = DEFAULT_ERROR_HANDLER {
        f(display, error_event)
    } else {
        panic!("No default error handler");
    }
}

#[no_mangle]
pub unsafe extern "C" fn check_other_wm(display: *mut Display) {
    let x_error_xlib = XSetErrorHandler(Some(x_error_start));
    DEFAULT_ERROR_HANDLER = x_error_xlib;

    XSelectInput(
        display,
        XDefaultRootWindow(display),
        SubstructureRedirectMask,
    );

    XSync(display, 0);
    XSetErrorHandler(Some(x_error));
    XSync(display, 0);
}

unsafe fn next_tiled(mut client: *mut Client) -> *mut Client {
    while !client.is_null() &&
        ((*client).is_floating == 1 || !(*client).is_visable())
    {
        client = (*client).next;
    }

    return client;
}

#[no_mangle]
pub unsafe extern "C" fn rust_monocle(monitor: *mut Monitor) {
    let monitor = &mut *monitor;

    let mut count = 0;
    let mut client = monitor.clients;
    while !client.is_null() {
        if (*client).is_visable() {
            count += 1;
        }
        client = (*client).next;
    }

    if count > 0 {
        // TODO(patrik): Find a better way to do this
        let s = format!("[{}]", count);
        let s = CString::new(s).unwrap();

        let bytes = s.to_bytes_with_nul();
        monitor.ltsymbol[..bytes.len()].clone_from_slice(bytes);
    }

    let mut client = next_tiled(monitor.clients);
    while !client.is_null() {
        let border_width = (*client).border_width;
        resize(
            client,
            monitor.wx,
            monitor.wy,
            monitor.ww - border_width * 2,
            monitor.wh - border_width * 2,
            0,
        );
        client = next_tiled((*client).next);
    }
}
