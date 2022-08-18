#pragma once

void hello_world_rust();

void print_monitor(Monitor *monitor);

void check_other_wm(Display *display);

void rust_monocle(Monitor *monitor);
void rust_tile(Monitor *monitor);

void rust_test(Drw *drw, Clr *scheme);

void rust_resize_bar_window(Display *display, Monitor *monitor);

int rust_draw_bar(Drw *drw, Monitor *monitor);

void rust_attach(Client *client);
void rust_attach_stack(Client *client);
void rust_detach(Client *client);
void rust_detach_stack(Client *client);

Client *rust_window_to_client(Window window, Monitor *monitor_list);

void rust_expose_event(XEvent *event);
void rust_focus_in_event(XEvent *event);
void rust_mapping_notify_event(XEvent *event);

