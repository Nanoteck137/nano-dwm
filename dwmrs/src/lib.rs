use x11::xlib::{
    Display, XSetErrorHandler, XErrorEvent, XSelectInput, XDefaultRootWindow,
    SubstructureRedirectMask, XSync, BadWindow, BadDrawable, BadMatch,
    BadAccess, Window, Visual, Colormap, Drawable, GC, XMoveResizeWindow,
};
use x11::xft::XftColor;
use std::ffi::{c_int, c_uint, c_uchar, c_char, c_float, c_void, CString, CStr};

const X_CONFIGURE_WINDOW: c_uchar = 12;
const X_GRAB_BUTTON: c_uchar = 28;
const X_GRAB_KEY: c_uchar = 33;
const X_SET_INPUT_FOCUS: c_uchar = 42;
const X_COPY_AREA: c_uchar = 62;
const X_POLY_SEGMENT: c_uchar = 66;
const X_POLY_TEXT_8: c_uchar = 74;
const X_POLY_FILL_RECTANGLE: c_uchar = 70;

const BAR_ITEM_WIDTH: u32 = 40;

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

    fn drw_setscheme(drw: *mut Drw, scheme: *mut XftColor);

    fn drw_rect(
        drw: *mut Drw,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        filled: c_int,
        invert: c_int,
    );

    fn drw_text(
        drw: *mut Drw,
        x: c_int,
        y: c_int,
        w: c_uint,
        h: c_uint,
        lpad: c_uint,
        text: *const c_char,
        invert: c_int,
    ) -> c_int;

    fn drw_fontset_getwidth(drw: *mut Drw, text: *const c_char) -> c_uint;

    fn drw_map(
        drw: *mut Drw,
        window: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
    );

    fn resizebarwin(monitor: *mut Monitor);

    fn systraytomon(monitor: *mut Monitor) -> *mut Monitor;
    fn getsystraywidth() -> c_uint;

    static scheme: *mut *mut XftColor;
    static bh: c_int;
    static lrpad: c_int;

    static selmon: *mut Monitor;
    static stext: [c_char; 256];
}

static TAGS: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

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

    fn full_width(&self) -> i32 {
        self.width + self.border_width * 2
    }

    fn full_height(&self) -> i32 {
        self.height + self.border_width * 2
    }
}

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

#[repr(C)]
pub struct Drw {
    width: c_int,
    height: c_int,
    display: *mut Display,
    screen: c_int,
    root: Window,
    visual: *mut Visual,
    depth: c_uint,
    cmap: Colormap,
    drawbale: Drawable,
    gc: GC,
    scheme: *mut XftColor,
    font: *mut c_void,
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

#[no_mangle]
pub unsafe extern "C" fn rust_tile(monitor: *mut Monitor) {
    let monitor = &*monitor;

    let mut count = 0;
    let mut client = monitor.clients;
    while !client.is_null() {
        if (*client).is_visable() {
            count += 1;
        }
        client = (*client).next;
    }

    if count == 0 {
        return;
    }

    let mw = if count > monitor.nmaster {
        if monitor.nmaster > 0 {
            (monitor.ww as f32 * monitor.mfact) as i32
        } else {
            0
        }
    } else {
        monitor.ww
    };

    let mut my = 0;
    let mut ty = 0;

    let mut index = 0;
    let mut client = next_tiled(monitor.clients);
    while !client.is_null() {
        let border_width = (*client).border_width;

        if index < monitor.nmaster {
            let h = (monitor.wh - my) / (count.min(monitor.nmaster) - index);
            resize(
                client,
                monitor.wx,
                monitor.wy + my,
                mw - (border_width * 2),
                h - (border_width * 2),
                0,
            );

            if my + (*client).full_height() < monitor.wh {
                my += (*client).full_height();
            }
        } else {
            let h = (monitor.wh - ty) / (count - index);
            resize(
                client,
                monitor.wx + mw,
                monitor.wy + ty,
                monitor.ww - mw - (border_width * 2),
                h - (border_width * 2),
                0,
            );

            if ty + (*client).full_height() < monitor.wh {
                ty += (*client).full_height();
            }
        }

        client = next_tiled((*client).next);
        index += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_resize_bar_window(
    display: *mut Display,
    monitor: *mut Monitor,
) {
    let monitor = &*monitor;

    let width = (*monitor).ww;
    // TODO(patrik): Systray
    // if (showsystray && m == systraytomon(m))
    // 	w -= getsystraywidth();

    XMoveResizeWindow(
        display,
        monitor.bar_window,
        monitor.wx,
        monitor.by,
        width as u32,
        bh as u32,
    );
}

#[no_mangle]
pub unsafe extern "C" fn rust_draw_bar(
    drw: *mut Drw,
    monitor_ptr: *mut Monitor,
) {
    let monitor = &*monitor_ptr;

    const SHOW_SYS_TRAY: bool = false;

    let systray_width =
        if SHOW_SYS_TRAY && monitor_ptr == systraytomon(monitor_ptr) {
            getsystraywidth()
        } else {
            0
        };

    let mut urg = 0;
    let mut client = monitor.clients;
    while !client.is_null() {
        if (*client).is_urgent > 0 {
            urg |= (*client).tags;
        }

        client = (*client).next;
    }

    if monitor_ptr == selmon {
        drw_setscheme(drw, *scheme.offset(0));

        let tw = drw_fontset_getwidth(drw, stext.as_ptr()) + lrpad as u32;
        let tw = tw as i32;
        let tw = tw - lrpad / 2 + 2;

        drw_text(
            drw,
            monitor.ww - tw - systray_width as i32,
            0,
            tw.try_into().unwrap(),
            bh.try_into().unwrap(),
            (lrpad / 2 - 2).try_into().unwrap(),
            stext.as_ptr(),
            0,
        );
    }

    resizebarwin(monitor_ptr);

    // TODO(patrik): Better way?
    let arrow = CString::new("\u{e0b0}").unwrap();
    let arrow_width =
        drw_fontset_getwidth(drw, arrow.as_ptr() as *const c_char);

    let mut x = 0;
    for (index, tag) in TAGS.iter().enumerate() {
        let tag = CString::new(*tag).unwrap();

        let selected =
            monitor.tagset[monitor.seltags as usize] & (1 << index) > 0;
        let mut next_selected = false;
        if (index + 1) < TAGS.len() {
            next_selected = monitor.tagset[monitor.seltags as usize] &
                (1 << (index + 1)) >
                0;
        }

        let selected_scheme = *scheme.offset(1);
        let normal_scheme = *scheme.offset(0);

        let mut normal_arrow_scheme = [
            *normal_scheme.offset(1),
            *normal_scheme.offset(1),
            *normal_scheme.offset(2),
        ];

        let mut selected_arrow_scheme = [
            *selected_scheme.offset(1),
            *normal_scheme.offset(1),
            *normal_scheme.offset(2),
        ];

        if next_selected {
            normal_arrow_scheme[0] = *normal_scheme.offset(1);
            normal_arrow_scheme[1] = *selected_scheme.offset(1);
        }

        // Tag Text
        drw_setscheme(
            drw,
            if selected {
                selected_scheme
            } else {
                normal_scheme
            },
        );

        let mut text_box_width = BAR_ITEM_WIDTH - arrow_width;
        let text_padding;
        if index == 0 {
            text_box_width += 5;
            text_padding = 10;
        } else {
            text_padding = 2;
        }

        // TODO(patrik): urg & 1 << i
        drw_text(
            drw,
            x,
            0,
            text_box_width,
            bh.try_into().unwrap(),
            text_padding,
            tag.as_ptr(),
            (urg & 1 << index) as i32,
        );

        // Arrow
        drw_setscheme(
            drw,
            if selected {
                selected_arrow_scheme.as_mut_ptr()
            } else {
                normal_arrow_scheme.as_mut_ptr()
            },
        );

        drw_text(
            drw,
            x + text_box_width as i32,
            0,
            arrow_width,
            bh.try_into().unwrap(),
            0,
            arrow.as_ptr(),
            0,
        );

        x += (arrow_width + text_box_width) as i32;
    }

    let w = drw_fontset_getwidth(
        drw,
        monitor.ltsymbol.as_ptr() as *const c_char,
    ) + lrpad as u32;
    drw_setscheme(drw, *scheme.offset(0));
    let s = CStr::from_ptr(monitor.ltsymbol.as_ptr() as *const c_char);
    let x = drw_text(
        drw,
        x - 8,
        0,
        w,
        bh.try_into().unwrap(),
        (lrpad / 2).try_into().unwrap(),
        monitor.ltsymbol.as_ptr() as *const c_char,
        0,
    );

    // let w = monitor.ww - tw - stw - x;

    // if ((w = m->ww - tw - stw - x) > bh) {
    // 	if (m->sel) {
    // 		drw_setscheme(drw, scheme[m == selmon ? SchemeSel : SchemeNorm]);
    // 		drw_text(drw, x, 0, w, bh, lrpad / 2, m->sel->name, 0);
    // 		if (m->sel->isfloating)
    // 			drw_rect(drw, x + boxs, boxs, boxw, boxw, m->sel->isfixed, 0);
    // 	} else {
    // 		drw_setscheme(drw, scheme[SchemeNorm]);
    // 		drw_rect(drw, x, 0, w, bh, 1, 1);
    // 	}
    // }
    //

    drw_map(
        drw,
        monitor.bar_window,
        0,
        0,
        monitor.ww.try_into().unwrap(),
        bh.try_into().unwrap(),
    );
}
