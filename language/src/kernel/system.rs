use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::runtime::Runtime;
use crate::value::Value;

pub fn load(runtime: &mut Runtime) {

    runtime.namespace("cons")
       .function("print", cons_print)
       .function("println", cons_println);

    runtime.namespace("file")
       .function("read", file_read)
       .function("write", file_write)
       .function("exists", file_exists);

    runtime.namespace("time")
       .function("sleep", time_sleep);

    runtime.namespace("window")
       .function("open", window_open)
       .function("clos", window_close);

    runtime.namespace("rand")
       .function("rdom", rand_rdom);

    // Прокачка окон (обработка их системных событий, перерисовка) на
    // каждой итерации цикла интерпретатора — независимо от @update/событий.
    runtime.register_tick(pump_windows);

    // Событие `@cons.imput(param){...}` — интерпретатор опрашивает эту
    // функцию на каждой итерации, но только если в скрипте есть такой обработчик.
    runtime.event_source("cons", "imput", cons_poll_input);

}

fn cons_print(args: Vec<Value>) -> Value {

    for value in args {

        print!("{}", value);

    }

    Value::Null

}

fn cons_println(args: Vec<Value>) -> Value {

    for value in args {

        print!("{}", value);

    }

    println!();

    Value::Null

}

fn time_sleep(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("time.sleep(ms)");

    }

    let ms = args[0].as_int();

    std::thread::sleep(

        std::time::Duration::from_millis(ms as u64)

    );

    Value::Null

}

fn file_exists(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("file.exists(path)");

    }

    let path = args[0].as_string();

    Value::Bool(

        std::path::Path::new(&path).exists()

    )

}

fn file_read(args: Vec<Value>) -> Value {

    if args.len() != 1 {

        panic!("file.read(path)");

    }

    let path = args[0].as_string();

    let text = std::fs::read_to_string(path)

        .unwrap_or_default();

    Value::String(text)

}

fn file_write(args: Vec<Value>) -> Value {

    if args.len() != 2 {

        panic!("file.write(path,text)");

    }

    let path = args[0].as_string();

    let text = args[1].as_string();

    std::fs::write(path, text)

        .expect("Ошибка записи файла");

    Value::Null

}

/// Простой генератор псевдослучайных чисел (xorshift64*), без внешних
/// зависимостей. Затравка — системное время + счётчик вызовов, чтобы
/// не повторяться при вызовах в один и тот же наносекундный момент.
fn next_random_u64() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};

    static CALLS: AtomicU64 = AtomicU64::new(0);
    let calls = CALLS.fetch_add(1, Ordering::Relaxed);

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut x = time ^ calls.wrapping_mul(0x9E3779B97F4A7C15) ^ 0x2545_F491_4F6C_DD1D;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// `rand.rdom(min, max)` — случайное целое число в диапазоне [min, max] включительно.
fn rand_rdom(args: Vec<Value>) -> Value {

    if args.len() != 2 {
        panic!("rand.rdom(min, max)");
    }

    let min = args[0].as_int();
    let max = args[1].as_int();

    if max < min {
        panic!("rand.rdom: max должен быть не меньше min");
    }

    let range = (max - min + 1) as u64;
    let n = next_random_u64() % range;

    Value::Int(min + n as i64)

}

/// Опрашивается интерпретатором: если пользователь ввёл строку в консоль —
/// возвращает её как значение события; если поток ввода закрыт (EOF) — None.
fn cons_poll_input() -> Option<Value> {

    let mut line = String::new();

    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None, // EOF — поток ввода закрыт
        Ok(_) => {
            let line = line.trim_end_matches(['\n', '\r']).to_string();
            Some(Value::String(line))
        }
        Err(_) => None,
    }

}

/// Настоящее окно (minifb). Оборачиваем в newtype и вручную помечаем Send:
/// minifb::Window не Send по умолчанию (хранит платформенные указатели),
/// но в этом проекте вообще нет многопоточности — интерпретатор целиком
/// однопоточный, и окна создаются/используются только из него. Если в
/// будущем в проект добавят потоки — это допущение нужно будет пересмотреть.
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

/// `window.open(xh, yh, x, y, name, sysid)` — открывает настоящее окно
/// (X11 на Linux, нативное окно на Windows). xh/yh: ширина/высота,
/// x/y: позиция на экране, name: заголовок, sysid: id окна (задаётся скриптом).
/// Возвращает true при успехе, паникует, если sysid уже занят.
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
        panic!("window.open: окно с sysid={} уже существует", sysid);
    }

    let mut window = minifb::Window::new(
        &name,
        width,
        height,
        minifb::WindowOptions::default(),
    ).unwrap_or_else(|err| panic!("window.open: не удалось открыть окно: {}", err));

    window.set_position(x as isize, y as isize);

    // Заливаем окно нейтральным серым фоном — рисование конкретных
    // пикселей/фигур из скрипта пока не реализовано, это следующий шаг.
    let buffer = vec![0x00303030u32; width * height];
    window
        .update_with_buffer(&buffer, width, height)
        .unwrap_or_else(|err| panic!("window.open: ошибка отрисовки: {}", err));

    println!(
        "[window] открыто окно #{} \"{}\" {}x{} в позиции ({}, {})",
        sysid, name, width, height, x, y
    );

    registry.insert(sysid, WindowHandle { window, width, height, buffer });

    Value::Bool(true)

}

/// `window.clos(sysid)` — закрыть окно по id. Возвращает true, если окно
/// существовало и было закрыто, иначе false.
fn window_close(args: Vec<Value>) -> Value {

    if args.len() != 1 {
        panic!("window.clos(sysid)");
    }

    let sysid = args[0].as_int();
    let existed = windows().lock().unwrap().remove(&sysid).is_some();

    if existed {
        println!("[window] закрыто окно #{}", sysid);
    } else {
        println!("[window] окно #{} не найдено", sysid);
    }

    Value::Bool(existed)

}

/// Вызывается интерпретатором каждую итерацию цикла: перерисовывает и
/// обрабатывает системные события всех открытых окон. Окна, закрытые
/// пользователем (крестик/Escape), удаляются из реестра автоматически.
/// Возвращает true, если после прокачки осталось хотя бы одно окно —
/// это не даёт программе завершиться, пока окно ещё открыто.
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