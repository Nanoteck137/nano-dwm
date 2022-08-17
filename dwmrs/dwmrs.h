#pragma once

void hello_world_rust();

void print_monitor(Monitor *monitor);

void check_other_wm(Display *display);

void rust_monocle(Monitor *monitor);
void rust_tile(Monitor *monitor);

void rust_test(Drw *drw, Clr *scheme);

void rust_resize_bar_window(Display *display, Monitor *monitor);

int rust_draw_bar(Drw *drw, Monitor *monitor);

