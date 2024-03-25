extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;

use sdl2::gfx::primitives::DrawRenderer;

use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiInput};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() -> Result<(), String> {

    // let mut input = String::new();

    // let mut midi_in = MidiInput::new("midir reading input").unwrap();
    // midi_in.ignore(Ignore::None);

    // // Get an input port (read from console if multiple are available)
    // let in_ports = midi_in.ports();
    // let in_port = match in_ports.len() {
    //     0 => return Err("no input port found".into()),
    //     1 => {
    //         println!(
    //             "Choosing the only available input port: {}",
    //             midi_in.port_name(&in_ports[0]).unwrap()
    //         );
    //         &in_ports[0]
    //     }
    //     _ => {
    //         println!("\nAvailable input ports:");
    //         for (i, p) in in_ports.iter().enumerate() {
    //             println!("{}: {}", i, midi_in.port_name(p).unwrap());
    //         }
    //         print!("Please select input port: ");
    //         stdout().flush().unwrap();
    //         let mut input = String::new();
    //         stdin().read_line(&mut input).unwrap();
    //         in_ports
    //             .get(input.trim().parse::<usize>().unwrap())
    //             .ok_or("invalid input port selected")?
    //     }
    // };

    // println!("\nOpening connection");
    // let in_port_name = midi_in.port_name(in_port).unwrap();

    // // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    // let _conn_in = midi_in.connect(
    //     in_port,
    //     "midir-read-input",
    //     move |stamp, message, _| {
    //         println!("{}: {:?} (len = {})", stamp, message, message.len());
    //     },
    //     (),
    // ).unwrap();

    // println!(
    //     "Connection open, reading input from '{}' (press enter to exit) ...",
    //     in_port_name
    // );

    // input.clear();
    // stdin().read_line(&mut input).unwrap(); // wait for next enter key press

    // println!("Closing connection");


    // return Ok(());








    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window(
            "rust-sdl2_gfx: draw line & FPSManager",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut lastx = 0;
    let mut lasty = 0;

    let mut events = sdl_context.event_pump()?;

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'main;
                    } else if keycode == Keycode::Space {
                        println!("space down");
                        for i in 0..400 {
                            canvas.pixel(i as i16, i as i16, 0xFF000FFu32)?;
                        }
                        canvas.present();
                    }
                }

                Event::MouseButtonDown { x, y, .. } => {
                    let color = pixels::Color::RGB(x as u8, y as u8, 255);
                    let _ = canvas.line(lastx, lasty, x as i16, y as i16, color);
                    lastx = x as i16;
                    lasty = y as i16;
                    println!("mouse btn down at ({},{})", x, y);
                    canvas.present();
                }

                _ => {}
            }
        }
    }

    Ok(())
}