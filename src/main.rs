extern crate sdl2;

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::HashMap;
use std::convert::From;
use std::convert::Into;
use std::rc::Rc;
use std::sync::Arc;

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


#[derive(Debug)]
enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug)]
enum Accidental{
    Sharp,
    Flat,
    Natural
}

#[derive(Debug, PartialEq)]
enum Clef {
    Sol,
    Fa
}

enum KeySignatureAccidental{
    Sharp,
    Flat,
}
struct KeySignature(KeySignatureAccidental, u8);
impl KeySignature{
    fn new(accidental: KeySignatureAccidental, nb: u8) -> KeySignature {
        let nb = nb.clamp(0, 7);
        KeySignature(accidental, nb)
    }
}

const ORDER_SIGNATURE_SHARP: [Pitch; 7] = [Pitch::F, Pitch::C, Pitch::G, Pitch::D, Pitch::A, Pitch::E, Pitch::B];
const ORDER_SIGNATURE_FLAT: [Pitch; 7] = [Pitch::B, Pitch::E, Pitch::A, Pitch::D, Pitch::G, Pitch::C, Pitch::F];

impl fmt::Display for Clef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Pitch {
    pub fn get_factor_gap(&self, clef: &Clef) -> i32{
        let mut r = match self{
            Self::A => 5,
            Self::B => 4,
            Self::C => 3,
            Self::D => 2,
            Self::E => 1,
            Self::F => 0,
            Self::G => -1,
            _ => 0
        };

        if *clef == Clef::Fa {
            r += 2;
        }

        r
    }
}

impl Distribution<Pitch> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Pitch {
        match rng.gen_range(0..=7) {
            0 => Pitch::A,
            1 => Pitch::B,
            2 => Pitch::C,
            3 => Pitch::D,
            4 => Pitch::E,
            5 => Pitch::F,
            _ => Pitch::G
        }
    }
}


//from -1 to 7 on my piano
#[derive(Debug)]
struct Octave(i32);
impl Octave {
    pub fn get_factor_gap(&self, clef: &Clef) -> i32{
        match clef {
            Clef::Sol => (4-self.0)*7,
            Clef::Fa => (3-self.0)*7,
        }
    }
}

#[derive(Debug)]
struct Note {
    pitch: Pitch,
    accidental: Option<Accidental>,
    octave: Octave,
    color: Color
}

impl Note {
    fn new(pitch: Pitch, accidental: Option<Accidental>, octave: Octave) -> Note{
        Note {
            pitch,
            accidental,
            octave,
            color: Color::BLACK
        }
    }
}

impl From<u8> for Note {
    fn from(value: u8) -> Self {
        //24 is C0
        let mut octave = (value as i32-24)/12;
        if (value as i32-24) < 0 { octave -= 1; }
        
        let remainder = (value as i32-24)%12;
        let (pitch, accidental) = match remainder {
            -3 => (Pitch::A, None),
            -2 => (Pitch::A, Some(Accidental::Sharp)),
            -1 => (Pitch::B, None),
            0 => (Pitch::C, None),
            1 => (Pitch::C, Some(Accidental::Sharp)),
            2 => (Pitch::D, None),
            3 => (Pitch::D, Some(Accidental::Sharp)),
            4 => (Pitch::E, None),
            5 => (Pitch::F, None),
            6 => (Pitch::F, Some(Accidental::Sharp)),
            7 => (Pitch::G, None),
            8 => (Pitch::G, Some(Accidental::Sharp)),
            9 => (Pitch::A, None),
            10 => (Pitch::A, Some(Accidental::Sharp)),
            11 => (Pitch::B, None),
            _ => panic!("remainder should be [-3 (on my piano) => 11]"),
        };
        Note::new(pitch, accidental, Octave(octave))
    }
}

struct Stave {
    pos: Point,
    width : i32,
    height : i32,
    //height between two consecutives notes
    //= radius of notes
    //gap*2 = gap between two lines
    gap : i32,
    clef: Clef,
    key_signature: Option<KeySignature>,
    notes: Vec<Note>,
}

impl Stave {
    #[allow(dead_code)]
    pub fn new(pos_y: i32, clef: Clef, key_signature: Option<KeySignature>) -> Stave {
        let width = (SCREEN_WIDTH as f32-(SCREEN_WIDTH as f32*0.2)) as i32;
        let height = 50;
        let gap = height/10;

        Stave{
            width,
            height,
            gap,
            pos: Point::new(((SCREEN_WIDTH as f32 - width as f32)/2. as f32) as i32, pos_y),
            notes: Vec::new(),
            key_signature,
            clef
        }
    }

    pub fn new_random(pos_y: i32, clef: Clef, key_signature: Option<KeySignature>) -> Stave {
        let mut s = Stave::new(pos_y, clef, key_signature);

        for _ in 0..15 {
            s.add_note(Note::new(rand::random(), None, Octave(4)));
        }
        s
    }

    pub fn add_note(&mut self, note: Note){
        self.notes.push(note);
    }

    pub fn draw(&self, canvas: &WindowCanvas){
        //draw lines
        for i in 0..5 {
            canvas.thick_line(self.pos.x as i16, (self.pos.y+(self.gap*2*i)as i32) as i16, (self.pos.x+self.width as i32) as i16, (self.pos.y+self.gap*2*i) as i16, 2, Color::BLACK).unwrap();
        }
        
        //draw clef
        let pos_clef = match self.clef{
            Clef::Sol => self.pos.y+self.height/2,
            Clef::Fa =>  self.pos.y+self.height/2-20,
        };
        canvas.string((self.pos.x-50) as i16, pos_clef as i16, &self.clef.to_string(), Color::BLACK).unwrap();
        

        

        //draw key_signature
        let gap_x = self.width/60;
        if let Some(key) = &self.key_signature{
            let s;
            let order;
            match key.0 {
                KeySignatureAccidental::Sharp => {s = '#'; order = ORDER_SIGNATURE_SHARP;},
                KeySignatureAccidental::Flat => {s = 'b'; order = ORDER_SIGNATURE_FLAT;},
            }
            
            for i in 0..key.1 {
                let y = self.pos.y + order[i as usize].get_factor_gap(&self.clef)*self.gap + 1 - self.gap;
                let x = self.pos.x + gap_x*i as i32;
                canvas.character(x as i16, y as i16, s, Color::BLACK).unwrap();
            }
        }
        
        let start_x = gap_x*8;
        let gap_x = self.width/15;
        //draw notes
        for (i, n) in self.notes.iter().enumerate() {
            let nb_factor_gap = n.pitch.get_factor_gap(&self.clef) + n.octave.get_factor_gap(&self.clef);
            let y = self.pos.y + nb_factor_gap*self.gap;
            let x = self.pos.x + start_x + gap_x*i as i32;
            canvas.filled_circle(x as i16, y as i16, self.gap as i16, n.color).unwrap();
            //draw help lines
            let help_line_width = (self.gap as f32*1.5) as i32;
            if nb_factor_gap <= -2 {
                for i_y in (2..=-nb_factor_gap).step_by(2){
                    let y = self.pos.y + i_y*-self.gap;
                    canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, Color::BLACK).unwrap();
                }
            }
            else if nb_factor_gap >= 10 {
                for i_y in (10..=nb_factor_gap).step_by(2){
                    let y = self.pos.y + i_y*self.gap;
                    canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, Color::BLACK).unwrap();
                }
            }
        }
    }
}


struct Game {
    staves: Vec<Stave>,
    current_note: usize,
}

impl Game {
    fn new() -> Game{
        let staves = vec![
            Stave::new_random(50, Clef::Sol, Some(KeySignature::new(KeySignatureAccidental::Sharp, 3))),
            Stave::new_random(150, Clef::Fa, Some(KeySignature::new(KeySignatureAccidental::Flat, 5))),
            Stave::new_random(250, Clef::Fa, None),
        ];

        Game {
            staves,
            current_note: 0
        }
    }

    fn note_pressed(&self, note: &Note){
        println!("note: {:?}", note);
    }

    fn draw(&self, canvas: &WindowCanvas){
        for s in &self.staves {
            s.draw(canvas);
        }

        // let x_tr = self.pos.x + self.current_note as i32*gap_x;
        // canvas.filled_trigon((x_tr-5) as i16, (self.pos.y-15) as i16, (x_tr+5) as i16, (self.pos.y-15) as i16, x_tr as i16, (self.pos.y-5) as i16, Color::BLACK).unwrap();
    }
}




fn main() -> Result<(), String> {

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

    println!("\nOpening connection");
    // let in_port_name = midi_in.port_name(in_port).unwrap();





    let game = Arc::new(Game::new());

    let callback = |_, message: &[u8], g: &mut Arc<Game>| {
        if message.len() == 3 {
            //0 is key released
            if message[2] != 0 {
                // println!("{:?}", message);
                let note = Note::from(message[1]);
                g.note_pressed(&note);
            }
        } 
    };

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        callback,
        game.clone(),
    ).unwrap();



    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window(
            "Train piano",
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
        
        game.draw(&canvas);
        canvas.string(20, 400, &fps_manager.get_frame_count().to_string(), Color::RGB(0, 0, 0)).unwrap();


        canvas.present();

        fps_manager.delay();
    }

    Ok(())
}