use chip8::Chip8;
use clap::{App, Arg};
use piston_window::*;
use std::fs::File;
use std::io::Read;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn get_chip8_key(key: Key) -> Option<u8> {
    match key {
        Key::X => Some(0),
        Key::D1 => Some(1),
        Key::D2 => Some(2),
        Key::D3 => Some(3),
        Key::Q => Some(4),
        Key::W => Some(5),
        Key::E => Some(6),
        Key::A => Some(7),
        Key::S => Some(8),
        Key::D => Some(9),
        Key::Z => Some(10),
        Key::C => Some(11),
        Key::D4 => Some(12),
        Key::R => Some(13),
        Key::F => Some(14),
        Key::V => Some(15),
        _ => None,
    }
}

fn main() {
    let matches = App::new("Chip-8 emulator")
        .version("0.1")
        .author("Robert Bartlensky")
        .about("A Chip-8 emulator.")
        .arg(
            Arg::with_name("rom")
                .short("r")
                .long("rom")
                .value_name("PATH")
                .help("Loads the specified ROM")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let rom = matches.value_of("rom").unwrap();
    let mut program: Vec<u8> = vec![];
    File::open(rom)
        .expect(&format!("Can't open file: '{}'", rom))
        .read_to_end(&mut program)
        .unwrap();
    let mut window: PistonWindow = WindowSettings::new("Chip8-emu", (640, 320))
        .fullscreen(true)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
    let settings = EventSettings::new();
    // 60 updates per second is enough for Chip8
    settings.ups(60);
    settings.bench_mode(true);
    window.get_event_settings().set_event_settings(settings);
    let mut chip = Chip8::new(program);
    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            window.draw_2d(&e, |c, g, _| {
                let screen = chip.screen();
                for j in 0..32 {
                    for i in 0..64 {
                        rectangle(
                            if (screen[j] >> (63 - i)) & 0x1 != 0 {
                                WHITE
                            } else {
                                BLACK
                            },
                            [i as f64 * 10.0, j as f64 * 10.0, 10.0, 10.0],
                            c.transform,
                            g,
                        );
                    }
                }
            });
        }
        if let Some(_) = e.update_args() {
            chip.step();
            chip.decrement_delay();
            chip.decrement_sound();
            if chip.sound() > 0 {
                // TODO: make noise
            }
        }
        if let Some(b) = e.press_args() {
            if let Button::Keyboard(key) = b {
                if let Some(k) = get_chip8_key(key) {
                    chip.press_key(k);
                }
            }
        }
        if let Some(b) = e.release_args() {
            if let Button::Keyboard(key) = b {
                if let Some(k) = get_chip8_key(key) {
                    chip.release_key(k);
                }
            }
        }
    }
}
