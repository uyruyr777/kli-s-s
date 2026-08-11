use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {
    runtime.namespace("window")
        .function("open", window_open)
        .function("clos", window_close)
        .function("clear", window_clear)
        .function("pixel", window_pixel)
        .function("rect", window_rect)
        .function("rectLine", window_rect_line)
        .function("line", window_line)
        .function("circle", window_circle)
        .function("circleLine", window_circle_line);

    runtime.register_tick(pump_windows);
}

struct WindowHandle {
    window: minifb::Window,
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

unsafe impl Send for WindowHandle {}

fn windows() -> &'static Mutex<HashMap<i64, WindowHandle>> {
    static WINDOWS: OnceLock<Mutex<HashMap<i64, WindowHandle>>> = OnceLock::new();
    WINDOWS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pack_rgb(r: i64, g: i64, b: i64) -> u32 {
    let r = r.clamp(0, 255) as u32;
    let g = g.clamp(0, 255) as u32;
    let b = b.clamp(0, 255) as u32;
    (r << 16) | (g << 8) | b
}

fn set_pixel(handle: &mut WindowHandle, x: i64, y: i64, color: u32) {
    if x < 0 || y < 0 {
        return;
    }

    let (x, y) = (x as usize, y as usize);

    if x >= handle.width || y >= handle.height {
        return;
    }

    handle.buffer[y * handle.width + x] = color;
}

fn with_window<F: FnOnce(&mut WindowHandle)>(sysid: i64, f: F) -> bool {
    let mut registry = windows().lock().unwrap();

    match registry.get_mut(&sysid) {
        Some(handle) => {
            f(handle);
            true
        }
        None => false,
    }
}

fn window_open(args: Vec<Value>) -> Value {
    if args.len() != 6 {
        panic!("window.open(xh, yh, x, y, name, sysid)");
    }

    let width = args[0].as_int().max(1) as usize;
    let height = args[1].as_int().max(1) as usize;
    let x = args[2].as_int();
    let y = args[3].as_int();
    let name = args[4].as_string();
    let sysid = args[5].as_int();

    let mut registry = windows().lock().unwrap();

    if registry.contains_key(&sysid) {
        panic!("window.open: a window with sysid={} already exists", sysid);
    }

    let mut window = minifb::Window::new(
        &name,
        width,
        height,
        minifb::WindowOptions::default(),
    ).unwrap_or_else(|err| panic!("window.open: failed to open window: {}", err));

    window.set_position(x as isize, y as isize);

    let buffer = vec![0x00303030u32; width * height];
    window
        .update_with_buffer(&buffer, width, height)
        .unwrap_or_else(|err| panic!("window.open: failed to draw: {}", err));

    println!(
        "[window] opened window #{} \"{}\" {}x{} at position ({}, {})",
        sysid, name, width, height, x, y
    );

    registry.insert(sysid, WindowHandle { window, width, height, buffer });

    Value::Bool(true)
}

fn window_close(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        panic!("window.clos(sysid)");
    }

    let sysid = args[0].as_int();
    let existed = windows().lock().unwrap().remove(&sysid).is_some();

    if existed {
        println!("[window] closed window #{}", sysid);
    } else {
        println!("[window] window #{} not found", sysid);
    }

    Value::Bool(existed)
}

fn window_clear(args: Vec<Value>) -> Value {
    if args.len() != 4 {
        panic!("window.clear(sysid, r, g, b)");
    }

    let sysid = args[0].as_int();
    let color = pack_rgb(args[1].as_int(), args[2].as_int(), args[3].as_int());

    let ok = with_window(sysid, |handle| {
        for pixel in handle.buffer.iter_mut() {
            *pixel = color;
        }
    });

    if !ok {
        panic!("window.clear: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_pixel(args: Vec<Value>) -> Value {
    if args.len() != 6 {
        panic!("window.pixel(sysid, x, y, r, g, b)");
    }

    let sysid = args[0].as_int();
    let x = args[1].as_int();
    let y = args[2].as_int();
    let color = pack_rgb(args[3].as_int(), args[4].as_int(), args[5].as_int());

    let ok = with_window(sysid, |handle| set_pixel(handle, x, y, color));

    if !ok {
        panic!("window.pixel: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_rect(args: Vec<Value>) -> Value {
    if args.len() != 8 {
        panic!("window.rect(sysid, x, y, w, h, r, g, b)");
    }

    let sysid = args[0].as_int();
    let x = args[1].as_int();
    let y = args[2].as_int();
    let w = args[3].as_int();
    let h = args[4].as_int();
    let color = pack_rgb(args[5].as_int(), args[6].as_int(), args[7].as_int());

    let ok = with_window(sysid, |handle| {
        for dy in 0..h.max(0) {
            for dx in 0..w.max(0) {
                set_pixel(handle, x + dx, y + dy, color);
            }
        }
    });

    if !ok {
        panic!("window.rect: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_rect_line(args: Vec<Value>) -> Value {
    if args.len() != 8 {
        panic!("window.rectLine(sysid, x, y, w, h, r, g, b)");
    }

    let sysid = args[0].as_int();
    let x = args[1].as_int();
    let y = args[2].as_int();
    let w = args[3].as_int();
    let h = args[4].as_int();
    let color = pack_rgb(args[5].as_int(), args[6].as_int(), args[7].as_int());

    let ok = with_window(sysid, |handle| {
        if w <= 0 || h <= 0 {
            return;
        }

        for dx in 0..w {
            set_pixel(handle, x + dx, y, color);
            set_pixel(handle, x + dx, y + h - 1, color);
        }

        for dy in 0..h {
            set_pixel(handle, x, y + dy, color);
            set_pixel(handle, x + w - 1, y + dy, color);
        }
    });

    if !ok {
        panic!("window.rectLine: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_line(args: Vec<Value>) -> Value {
    if args.len() != 8 {
        panic!("window.line(sysid, x1, y1, x2, y2, r, g, b)");
    }

    let sysid = args[0].as_int();
    let x0 = args[1].as_int();
    let y0 = args[2].as_int();
    let x1 = args[3].as_int();
    let y1 = args[4].as_int();
    let color = pack_rgb(args[5].as_int(), args[6].as_int(), args[7].as_int());

    let ok = with_window(sysid, |handle| {
        let mut x0 = x0;
        let mut y0 = y0;

        let dx = (x1 - x0).abs();
        let sx: i64 = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy: i64 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            set_pixel(handle, x0, y0, color);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;

            if e2 >= dy {
                err += dy;
                x0 += sx;
            }

            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    });

    if !ok {
        panic!("window.line: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_circle(args: Vec<Value>) -> Value {
    if args.len() != 7 {
        panic!("window.circle(sysid, cx, cy, radius, r, g, b)");
    }

    let sysid = args[0].as_int();
    let cx = args[1].as_int();
    let cy = args[2].as_int();
    let radius = args[3].as_int().max(0);
    let color = pack_rgb(args[4].as_int(), args[5].as_int(), args[6].as_int());

    let ok = with_window(sysid, |handle| {
        let r2 = radius * radius;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 {
                    set_pixel(handle, cx + dx, cy + dy, color);
                }
            }
        }
    });

    if !ok {
        panic!("window.circle: no window with sysid={}", sysid);
    }

    Value::Null
}

fn window_circle_line(args: Vec<Value>) -> Value {
    if args.len() != 7 {
        panic!("window.circleLine(sysid, cx, cy, radius, r, g, b)");
    }

    let sysid = args[0].as_int();
    let cx = args[1].as_int();
    let cy = args[2].as_int();
    let radius = args[3].as_int().max(0);
    let color = pack_rgb(args[4].as_int(), args[5].as_int(), args[6].as_int());

    let ok = with_window(sysid, |handle| {
        let mut x = radius;
        let mut y = 0i64;
        let mut err = 0i64;

        while x >= y {
            set_pixel(handle, cx + x, cy + y, color);
            set_pixel(handle, cx + y, cy + x, color);
            set_pixel(handle, cx - y, cy + x, color);
            set_pixel(handle, cx - x, cy + y, color);
            set_pixel(handle, cx - x, cy - y, color);
            set_pixel(handle, cx - y, cy - x, color);
            set_pixel(handle, cx + y, cy - x, color);
            set_pixel(handle, cx + x, cy - y, color);

            y += 1;

            if err <= 0 {
                err += 2 * y + 1;
            }

            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    });

    if !ok {
        panic!("window.circleLine: no window with sysid={}", sysid);
    }

    Value::Null
}

fn pump_windows() -> bool {
    let mut registry = windows().lock().unwrap();

    registry.retain(|_, handle| {
        let still_open = handle.window.is_open()
            && !handle.window.is_key_down(minifb::Key::Escape);

        if still_open {
            let _ = handle
                .window
                .update_with_buffer(&handle.buffer, handle.width, handle.height);
        }

        still_open
    });

    !registry.is_empty()
}
