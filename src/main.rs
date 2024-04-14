extern crate sdl2;



use std::sync::{Arc, Mutex};

use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::gfx::primitives::DrawRenderer;

use sdl2::pixels::Color;



use midir::{Ignore, MidiInput};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

mod game;
mod music;
use crate::game::Game;

fn main() -> Result<(), String> {

    let mut _conn_in;

    let mut asking_midi_port = true;

    let btn_pos_y = 100;
    let btn_size_y = 50;

    let game = Arc::new(Mutex::new(Game::new(SCREEN_WIDTH)));

    let callback = |_, message: &[u8], g: &mut Arc<Mutex<Game>>| {
        if message.len() == 3 {
            g.lock().unwrap().parse_midi_message(message);
        }
    };

        

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

                // Event::KeyDown {
                //     keycode: Some(keycode),
                //     ..
                // } => {
                //     if keycode == Keycode::Escape {
                //         break 'main;
                //     }
                // }

                Event::MouseButtonDown {  y, .. } => {
                    if asking_midi_port {
                        let mut midi_in = MidiInput::new("midir reading input").unwrap();
                        midi_in.ignore(Ignore::None);

                        // println!("mouse btn down at ({},{})", x, y);

                        let i = (y-btn_pos_y)/btn_size_y;
                        let in_ports = midi_in.ports();
                        if let Some(in_port) = in_ports.get(i as usize){
                            _conn_in = midi_in.connect(in_port, "read-input", callback, game.clone()).unwrap();
                        }

                        asking_midi_port = false;
                    }
                }

                _ => {}
            }
        }

        //logic

        //render
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        if asking_midi_port {
            let mut midi_in = MidiInput::new("midir reading input").unwrap();
            midi_in.ignore(Ignore::None);

            let in_ports = midi_in.ports();
            if in_ports.len() == 0 {
                canvas.string(20, 400, "No midi port", Color::RGB(0, 0, 0)).unwrap();
            }
            else{
                for (i, p) in in_ports.iter().enumerate() {
                    let s = format!("{}: {}", i, midi_in.port_name(p).unwrap());
                    canvas.string(20, (btn_pos_y+i as i32*btn_size_y) as i16, &s, Color::RGB(0, 0, 0)).unwrap();
                }
            }
        }
        else{
            game.lock().unwrap().draw(&canvas);
        }
        
        // canvas.string(20, 400, &fps_manager.get_frame_count().to_string(), Color::RGB(0, 0, 0)).unwrap();
        canvas.present();
        

        fps_manager.delay();
    }

    Ok(())
}

