extern crate sdl2;

use std::convert::Into;
use std::sync::{Arc, Mutex};

use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiInput};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

mod game;
mod music;
use crate::game::Game;

fn main() -> Result<(), String> {


    let game = Arc::new(Mutex::new(Game::new(SCREEN_WIDTH)));




    let mut midi_in = MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            in_ports
                .get(input.trim().parse::<usize>().unwrap())
                .ok_or("invalid input port selected")?
        }
    };

    let callback = |_, message: &[u8], g: &mut Arc<Mutex<Game>>| {
        if message.len() == 3 {
            g.lock().unwrap().parse_midi_message(message);
        }
    };

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in
        .connect(in_port, "midir-read-input", callback, game.clone())
        .unwrap();





        

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("Train piano", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60).unwrap();

    let mut events = sdl_context.event_pump()?;
    'main: loop {
        //events
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if keycode == Keycode::Escape {
                        break 'main;
                    }
                }

                // Event::MouseButtonDown { x, y, .. } => {
                //     println!("mouse btn down at ({},{})", x, y);
                // }
                _ => {}
            }
        }

        //logic

        //render
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        game.lock().unwrap().draw(&canvas);
        // canvas.string(20, 400, &fps_manager.get_frame_count().to_string(), Color::RGB(0, 0, 0)).unwrap();

        canvas.present();

        fps_manager.delay();
    }

    Ok(())
}
