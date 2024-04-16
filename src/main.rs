extern crate sdl2;
use sdl2::{event::Event, keyboard::Scancode};
use sdl2::gfx::framerate::FPSManager;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;

use midir::{Ignore, MidiInput};

use std::sync::{Arc, Mutex};

mod game;
mod music;
use crate::game::Game;



const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;


fn main() -> Result<(), String> {

    let mut _conn_in;
    let mut midi_in = Some(MidiInput::new("midir reading input").unwrap());
    midi_in.as_mut().unwrap().ignore(Ignore::None);



    let btn_pos_y = 70;
    let btn_size_y = 30;

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

                Event::KeyDown {  scancode, .. } => {
                    if let Some(sc) = scancode {
                        let index = match sc {
                            Scancode::Num0 | Scancode::Kp0 => Some(0),
                            Scancode::Num1 | Scancode::Kp1 => Some(1),
                            Scancode::Num2 | Scancode::Kp2 => Some(2),
                            Scancode::Num3 | Scancode::Kp3 => Some(3),
                            Scancode::Num4 | Scancode::Kp4 => Some(4),
                            Scancode::Num5 | Scancode::Kp5 => Some(5),
                            Scancode::Num6 | Scancode::Kp6 => Some(6),
                            Scancode::Num7 | Scancode::Kp7 => Some(7),
                            Scancode::Num8 | Scancode::Kp8 => Some(8),
                            Scancode::Num9 | Scancode::Kp9 => Some(9),
                            _ => None
                        };
                        
                        if let Some(i) = index {
                            let in_ports = midi_in.as_ref().unwrap().ports();
                            if let Some(in_port) = in_ports.get(i as usize){
                                _conn_in = midi_in.take().unwrap().connect(in_port, "read-input", callback, game.clone()).unwrap();
                            }
                        }
                    }
                }

                Event::MouseButtonDown {  y, .. } => {
                    if midi_in.is_some() {
                        // println!("mouse btn down at ({},{})", x, y);
                        let i = (y-btn_pos_y)/btn_size_y;
                        
                        let in_ports = midi_in.as_ref().unwrap().ports();
                        if let Some(in_port) = in_ports.get(i as usize){
                            _conn_in = midi_in.take().unwrap().connect(in_port, "read-input", callback, game.clone()).unwrap();
                        }
                    }
                }

                _ => {}
            }
        }

        //logic

        //render
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        if midi_in.is_some() {
            canvas.string(20, 20, "Select midi port (click on text or press key) :", Color::RGB(0, 0, 0)).unwrap();

            let in_ports = midi_in.as_ref().unwrap().ports();
            if in_ports.len() == 0 {
                canvas.string(20, btn_pos_y as i16, "No midi port", Color::RGB(0, 0, 0)).unwrap();
            }
            else{
                for (i, p) in in_ports.iter().enumerate() {
                    let s = format!("{}: {}", i, midi_in.as_ref().unwrap().port_name(p).unwrap());
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

