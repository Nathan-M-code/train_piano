extern crate sdl2;

use sdl2::gfx::framerate::FPSManager;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::{event::Event, sys::Window};
use sdl2::keyboard::Keycode;
use sdl2::pixels::{self, Color};

use sdl2::gfx::primitives::DrawRenderer;

use std::fmt;
use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiInput};

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;


enum Pitch {
    A,
    ASharp,
    AFlat,
    B,
    BSharp,
    BFlat,
    C,
    CSharp,
    CFlat,
    D,
    DSharp,
    DFlat,
    E,
    ESharp,
    EFlat,
    F,
    FSharp,
    FFlat,
    G,
    GSharp,
    GFlat,
}

#[derive(Debug, PartialEq)]
enum Clef {
    Sol,
    Fa
}

impl fmt::Display for Clef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Pitch {
    pub fn get_factor_height(&self, clef: &Clef) -> i32{
        let mut r = match self{
            Self::A => 2,
            Self::B => 1,
            Self::C => 0,
            Self::D => -1,
            Self::E => -2,
            Self::F => -3,
            Self::G => -4,
            _ => 0
        };

        if *clef == Clef::Fa {
            r += 2;
        }

        r
    }
}

struct Note {
    pitch: Pitch,
    octave: i32,
    color: Color
}

impl Note {
    fn new(pitch: Pitch, octave: i32) -> Note{
        Note {
            pitch,
            octave,
            color: Color::BLACK
        }
    }
}

struct Stave {
    pos: Point,
    width : i32,
    height : i32,
    gap : i32,
    clef: Clef,
    notes: Vec<Note>,
    current_note: usize,
}

impl Stave {
    pub fn new(clef: Clef) -> Stave {
        let width = (SCREEN_WIDTH as f32-(SCREEN_WIDTH as f32*0.2)) as i32;
        let height = 50;
        let gap = height/5;

        Stave{
            width,
            height,
            gap,
            pos: Point::new(((SCREEN_WIDTH as f32 - width as f32)/2. as f32) as i32, 20),
            notes: Vec::new(),
            current_note: 0,
            clef
        }
    }

    pub fn add_note(&mut self, note: Note){
        self.notes.push(note);
    }

    pub fn draw(&self, canvas: &WindowCanvas){
        let gap_x = self.width/15;

        let x_tr = self.pos.x + self.current_note as i32*gap_x;
        canvas.filled_trigon((x_tr-5) as i16, (self.pos.y-15) as i16, (x_tr+5) as i16, (self.pos.y-15) as i16, x_tr as i16, (self.pos.y-5) as i16, Color::BLACK).unwrap();

        for i in 0..5 {
            canvas.thick_line(self.pos.x as i16, (self.pos.y+(self.gap*i)as i32) as i16, (self.pos.x+self.width as i32) as i16, (self.pos.y+self.gap*i) as i16, 2, Color::BLACK).unwrap();
        }
        
        let pos_clef = match self.clef{
            Clef::Sol => self.pos.y+self.height/2,
            Clef::Fa =>  self.pos.y+self.height/2-20,
        };
        canvas.string((self.pos.x-50) as i16, pos_clef as i16, &self.clef.to_string(), Color::BLACK).unwrap();
        
        for (i, n) in self.notes.iter().enumerate() {
            let y = (self.pos.y as f32 + self.gap as f32*1.5)
                        +(n.pitch.get_factor_height(&self.clef)*self.gap/2) as f32
                        +((n.octave-4) * self.gap) as f32*3.5;
            canvas.filled_circle(self.pos.x as i16 + (gap_x*i as i32) as i16, y as i16, (self.gap/2) as i16, n.color).unwrap();
        }
    }
}

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

    
    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60).unwrap();

    let mut stave = Stave::new(Clef::Fa);
    stave.add_note(Note::new(Pitch::A, 4));
    stave.add_note(Note::new(Pitch::B, 4));
    stave.add_note(Note::new(Pitch::C, 4));
    stave.add_note(Note::new(Pitch::D, 4));
    stave.add_note(Note::new(Pitch::E, 4));
    stave.add_note(Note::new(Pitch::F, 4));
    stave.add_note(Note::new(Pitch::G, 4));
    
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
        canvas.set_draw_color(Color::RGB(255,255,255));
        canvas.clear();
        
        stave.draw(&canvas);
        canvas.string(20, 400, &fps_manager.get_frame_count().to_string(), Color::RGB(0, 0, 0)).unwrap();


        canvas.present();

        fps_manager.delay();
    }

    Ok(())
}