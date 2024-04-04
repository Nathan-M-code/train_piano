extern crate sdl2;

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::HashMap;
use std::convert::From;
use std::convert::Into;
use std::ops::DerefMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

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


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Accidental{
    Sharp,
    Flat,
    Natural
}

impl Distribution<Accidental> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Accidental {
        match rng.gen_range(0..=2) {
            0 => Accidental::Flat,
            1 => Accidental::Natural,
            _ => Accidental::Sharp,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

    fn is_pitch_inside(&self, p: Pitch) -> bool {
        match self.0 {
            KeySignatureAccidental::Sharp => {
                for i in 0..self.1{
                    if ORDER_SIGNATURE_SHARP[i as usize] == p { return true; }
                }
                return false;
            },
            KeySignatureAccidental::Flat => {
                for i in 0..self.1{
                    if ORDER_SIGNATURE_FLAT[i as usize] == p { return true; }
                }
                return false;
            },
        }
        false
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
            Self::A => -5,
            Self::B => -6,
            Self::C => 0,
            Self::D => -1,
            Self::E => -2,
            Self::F => -3,
            Self::G => -4
        };

        if *clef == Clef::Fa {
            r += 2;
        }

        r+3
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Octave(i32);
impl Octave {
    pub fn get_factor_gap(&self, clef: &Clef) -> i32{
        match clef {
            Clef::Sol => (4-self.0)*7,
            Clef::Fa => (3-self.0)*7,
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

//Note is same independently of its color
impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.pitch == other.pitch && self.accidental == other.accidental && self.octave == other.octave
    }
}
impl Eq for Note {}

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
    size: Point,
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
    pub fn new(
        pos: Point,
        size: Point,
        clef: Clef,
        key_signature: Option<KeySignature>
    ) -> Stave {
        let gap = size.y/10;

        Stave{
            pos,
            size,
            gap,
            notes: Vec::new(),
            key_signature,
            clef
        }
    }

    pub fn new_random(
        pos: Point,
        size: Point,
        clef: Clef,
        key_signature: Option<KeySignature>,
        nb_note: usize,
    ) -> Stave {
        let mut s = Stave::new(pos, size, clef, key_signature);

        for _ in 0..nb_note {
            s.add_note(Note::new(rand::random(), rand::random(), Octave(4)));
        }
        s
    }

    pub fn add_note(&mut self, note: Note){
        self.notes.push(note);
    }

    pub fn draw(&self, canvas: &WindowCanvas){
        //draw lines
        for i in 0..5 {
            canvas.thick_line(self.pos.x as i16, (self.pos.y+(self.gap*2*i)as i32) as i16, (self.pos.x+self.size.x as i32) as i16, (self.pos.y+self.gap*2*i) as i16, 2, Color::BLACK).unwrap();
        }
        
        //draw clef
        let pos_clef = match self.clef{
            Clef::Sol => self.pos.y+self.size.y/2,
            Clef::Fa =>  self.pos.y+self.size.y/2-20,
        };
        canvas.string((self.pos.x-50) as i16, pos_clef as i16, &self.clef.to_string(), Color::BLACK).unwrap();
        

        

        //draw key_signature
        let small_gap_x = self.size.x/60;
        if let Some(key) = &self.key_signature{
            let s;
            let order;
            match key.0 {
                KeySignatureAccidental::Sharp => {s = '#'; order = ORDER_SIGNATURE_SHARP;},
                KeySignatureAccidental::Flat => {s = 'b'; order = ORDER_SIGNATURE_FLAT;},
            }
            
            for i in 0..key.1 {
                let y = self.pos.y + order[i as usize].get_factor_gap(&self.clef)*self.gap + 1 - self.gap;
                let x = self.pos.x + small_gap_x*i as i32;
                canvas.character(x as i16, y as i16, s, Color::BLACK).unwrap();
            }
        }
        
        let start_x = small_gap_x*8;
        let gap_x = self.size.x/15;
        //draw notes
        for (i, n) in self.notes.iter().enumerate() {
            let nb_factor_gap = n.pitch.get_factor_gap(&self.clef) + n.octave.get_factor_gap(&self.clef);
            let y = self.pos.y + nb_factor_gap*self.gap;
            let x = self.pos.x + start_x + gap_x*i as i32;
            canvas.filled_circle(x as i16, y as i16, self.gap as i16, n.color).unwrap();

            //draw accidental
            if let Some(ks) = &self.key_signature {
                // if let Some(acci) = &n.accidental {
                //     match acci {
                //         Accidental::Sharp => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, '#', n.color).unwrap(),
                //         Accidental::Flat => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'b', n.color).unwrap(),
                //         Accidental::Natural => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'n', n.color).unwrap(),
                //     }
                // }
                // if !ks.is_pitch_inside(n.pitch){
                // }
            }
            //no key-sign, draw all accidental
            else{
                if let Some(acci) = &n.accidental {
                    match acci {
                        Accidental::Sharp => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, '#', n.color).unwrap(),
                        Accidental::Flat => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'b', n.color).unwrap(),
                        Accidental::Natural => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'n', n.color).unwrap(),
                    }
                }
            }
            
            
            //draw help lines
            let help_line_width = (self.gap as f32*1.5) as i32;
            if nb_factor_gap <= -2 {
                for i_y in (2..=-nb_factor_gap).step_by(2){
                    let y = self.pos.y + i_y*-self.gap;
                    canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, n.color).unwrap();
                }
            }
            else if nb_factor_gap >= 10 {
                for i_y in (10..=nb_factor_gap).step_by(2){
                    let y = self.pos.y + i_y*self.gap;
                    canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, n.color).unwrap();
                }
            }
        }
    }
}


struct Game {
    staves: Vec<Stave>,
    current_note: usize,
    pressed_note: Option<Note>,
}

impl Game {
    fn new() -> Game{
        let width = (SCREEN_WIDTH as f32-(SCREEN_WIDTH as f32*0.2)) as i32;
        let height = 50;
        let size = Point::new(width, height);
        let pos = Point::new(((SCREEN_WIDTH as f32 - width as f32)/2. as f32) as i32, 50);

        let mut rng = rand::thread_rng();

        let mut staves = Vec::new();
        for i in 0..=3 {
            let pos_stave = Point::new(pos.x, pos.y+i*(height+40));
            staves.push(Stave::new_random(pos_stave, size, Clef::Sol, None, rng.gen_range(1..14)));
        }

        staves.get_mut(0).unwrap().notes.get_mut(0).unwrap().color = Color::GRAY;

        Game {
            staves,
            current_note: 0,
            pressed_note: None,
        }
    }

    fn note_pressed(&mut self, note_pressed: &Note){
        println!("note_pressed: {:?}", note_pressed);
        self.pressed_note = Some(*note_pressed);
        
        let searched_note = self.staves.get_mut(0).unwrap().notes.get_mut(self.current_note).unwrap();
        println!("searched_note: {:?}", searched_note);
        
        if searched_note == note_pressed {
            searched_note.color = Color::GREEN;

            self.current_note += 1;
            if self.current_note == self.staves.get_mut(0).unwrap().notes.len() {
                self.staves.remove(0);
                self.current_note = 0;
            }
            //set the searched note GREY
            else{
                self.staves.get_mut(0).unwrap().notes.get_mut(self.current_note).unwrap().color = Color::GRAY;
            }
        }
        
        else{
            searched_note.color = Color::RED;
        }
    }
    
    fn note_released(&mut self, note_released: &Note){
        // println!("note_released: {:?}", note_released);
        // println!("self.pressed_note: {:?}", self.pressed_note);

        if self.pressed_note.as_ref() == Some(note_released){
            self.pressed_note = None;
            
            let searched_note = self.staves.get_mut(0).unwrap().notes.get_mut(self.current_note).unwrap();
            searched_note.color = Color::GRAY;
        }
    }

    fn draw(&self, canvas: &WindowCanvas){
        for s in &self.staves {
            s.draw(canvas);
        }
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





    let game = Arc::new(Mutex::new(Game::new()));

    let callback = |_, message: &[u8], g: &mut Arc<Mutex<Game>>| {
        if message.len() == 3 {
            // println!("{:?}", message);
            let note = Note::from(message[1]);
            //0 is key released
            if message[2] == 0 {
                g.lock().unwrap().note_released(&note);
            }else{
                g.lock().unwrap().note_pressed(&note);
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
        
        game.lock().unwrap().draw(&canvas);
        canvas.string(20, 400, &fps_manager.get_frame_count().to_string(), Color::RGB(0, 0, 0)).unwrap();


        canvas.present();

        fps_manager.delay();
    }

    Ok(())
}