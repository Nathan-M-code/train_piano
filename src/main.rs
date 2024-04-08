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


//A-1 on my piano is 21
//C0 = 24
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Semitone(u8);

impl Semitone {
    fn new_from(n: &Note) -> Semitone{
        //A-1 on my piano is 21
        //C0 = 24
        let mut v = 24;
        v += n.octave.0 * 12;
        v += n.pitch.get_semitone() as i32;
        if let Some(a) = n.accidental {
            v += a.get_semitone() as i32;
        }
        Semitone(v as u8)
    }
}



#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Accidental{
    Sharp,
    Flat,
    Natural
}

impl Accidental {
    fn get_semitone(&self) -> i8 {
        match self{
            Self::Sharp => 1,
            Self::Flat => -1,
            Self::Natural => 0,
        }
    }
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

#[derive(Debug, PartialEq, Clone, Copy)]
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
    }

    fn accidental_match(&self, a: Accidental) -> bool {
        if self.0 == KeySignatureAccidental::Sharp && a == Accidental::Sharp { return true; }
        if self.0 == KeySignatureAccidental::Flat && a == Accidental::Flat { return true; }
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
    fn get_semitone(&self) -> u8 {
        match self{
            Self::A => 9,
            Self::B => 11,
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7
        }
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Octave(i32);
impl Octave {
    pub fn get_factor_gap(&self, clef: &Clef) -> i32{
        match clef {
            Clef::Sol => (4-self.0)*7,
            Clef::Fa => (3-self.0)*7,
        }
    }
}


#[derive(Debug, Clone, Copy, Hash)]
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
        Semitone::new_from(&self) == Semitone::new_from(&other)
    }
}
impl Eq for Note {}



struct Measure{
    notes: Vec<Note>
}

impl Measure{
    fn new(notes: Vec<Note>) -> Measure{
        Measure{
            notes
        }
    }
}

struct Stave {
    x_pos: i32,
    size: Point,
    //height between two consecutives notes
    //= radius of notes
    //gap*2 = gap between two lines
    gap : i32,
    clef: Clef,
    key_signature: Option<KeySignature>,
    measures: Vec<Measure>,
}

impl Stave {
    #[allow(dead_code)]
    pub fn new(
        x_pos: i32,
        size: Point,
        clef: Clef,
        key_signature: Option<KeySignature>
    ) -> Stave {
        let gap = size.y/10;

        Stave{
            x_pos,
            size,
            gap,
            measures: Vec::new(),
            key_signature,
            clef
        }
    }

    pub fn new_random(
        x_pos: i32,
        size: Point,
        clef: Clef,
        key_signature: Option<KeySignature>,
    ) -> Stave {
        let mut s = Stave::new(x_pos, size, clef, key_signature);
        s.add_measure(Measure::new(vec![
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
        ]));
        s.add_measure(Measure::new(vec![
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
        ]));
        s.add_measure(Measure::new(vec![
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
            Note::new(rand::random(), rand::random(), Octave(3)),
        ]));
        s
    }

    pub fn add_measure(&mut self, m: Measure){
        self.measures.push(m);
    }

    pub fn draw(&self, y_pos: i32, canvas: &WindowCanvas){
        let pos = Point::new(self.x_pos, y_pos);
        //draw lines
        for i in 0..5 {
            canvas.thick_line(pos.x as i16, (pos.y+(self.gap*2*i)as i32) as i16, (pos.x+self.size.x as i32) as i16, (pos.y+self.gap*2*i) as i16, 2, Color::BLACK).unwrap();
        }
        
        //draw clef
        let pos_clef = match self.clef{
            Clef::Sol => pos.y+self.size.y/2,
            Clef::Fa =>  pos.y+self.size.y/2-20,
        };
        canvas.string(pos.x as i16, pos_clef as i16, &self.clef.to_string(), Color::BLACK).unwrap();

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
                let y = pos.y + order[i as usize].get_factor_gap(&self.clef)*self.gap + 1 - self.gap;
                let x = pos.x + small_gap_x*i as i32;
                canvas.character(x as i16, y as i16, s, Color::BLACK).unwrap();
            }
        }
        
        let mut current_x = pos.x + small_gap_x*8;
        let gap_x = self.size.x/15;
        //draw measures
        for m in self.measures.iter(){
            let mut previous_accidentals : HashMap<Pitch, Accidental> = HashMap::new();


            //draw notes
            for n in m.notes.iter() {
                let nb_factor_gap = n.pitch.get_factor_gap(&self.clef) + n.octave.get_factor_gap(&self.clef);
                let y = pos.y + nb_factor_gap*self.gap;
                let x = current_x;
                canvas.filled_circle(x as i16, y as i16, self.gap as i16, n.color).unwrap();
    
                //draw accidental
                let mut draw_acci = true;

                if let Some(ks) = &self.key_signature {
                    
                }
                //no key-sign, draw all # and b
                else{
                    if let Some(acci) = previous_accidentals.get(&n.pitch) {
                        if n.accidental.is_some(){
                            if *acci == n.accidental.unwrap() {
                                draw_acci = false;
                            }else{
                                todo!("blablabla");
                            }
                        }
                    }
                }

                if draw_acci {
                    match n.accidental {
                        Some(Accidental::Sharp) => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, '#', n.color).unwrap(),
                        Some(Accidental::Flat) => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'b', n.color).unwrap(),
                        None | Some(Accidental::Natural) => canvas.character((x-small_gap_x-2) as i16, (y-4) as i16, 'n', n.color).unwrap(),
                    }
                }
                
                
                //draw help lines
                let help_line_width = (self.gap as f32*1.5) as i32;
                if nb_factor_gap <= -2 {
                    for i_y in (2..=-nb_factor_gap).step_by(2){
                        let y = pos.y + i_y*-self.gap;
                        canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, n.color).unwrap();
                    }
                }
                else if nb_factor_gap >= 10 {
                    for i_y in (10..=nb_factor_gap).step_by(2){
                        let y = pos.y + i_y*self.gap;
                        canvas.thick_line((x-help_line_width) as i16, y as i16, (x+help_line_width) as i16 , y as i16, 2, n.color).unwrap();
                    }
                }

                current_x += gap_x;
            }
            
            //draw measures separating lines
            canvas.thick_line(current_x as i16, pos.y as i16, current_x as i16, (pos.y+self.gap*8) as i16, 2, Color::BLACK).unwrap();
            current_x += gap_x;
        }
    }
}


struct Game {
    size_stave: Point,
    x_pos_stave: i32,
    staves: Vec<Stave>,
    current_measure_note: (usize, usize),
    pressed_semitone: Option<Semitone>,
}

impl Game {
    fn new() -> Game{
        let width = (SCREEN_WIDTH as f32-(SCREEN_WIDTH as f32*0.1)) as i32;
        let height = 50;
        let size_stave = Point::new(width, height);
        let x_pos_stave = ((SCREEN_WIDTH as f32 - width as f32)/2. as f32) as i32;

        let mut rng = rand::thread_rng();

        let mut staves = Vec::new();
        for i in 0..4 {
            staves.push(Stave::new_random(x_pos_stave, size_stave, Clef::Sol, Some(KeySignature(KeySignatureAccidental::Sharp, 3))));
        }

        let current_measure_note = (0,0);
        staves.get_mut(0).unwrap().measures.get_mut(current_measure_note.0).unwrap().notes.get_mut(current_measure_note.1).unwrap().color = Color::GRAY;

        Game {
            size_stave,
            x_pos_stave,
            staves,
            current_measure_note,
            pressed_semitone: None,
        }
    }

    fn pressed_semitone(&mut self, pressed_semitone: &Semitone){
        println!("pressed_semitone: {:?}", pressed_semitone);
        self.pressed_semitone = Some(*pressed_semitone);
        
        let searched_note = self.staves.get_mut(0).unwrap().measures.get_mut(self.current_measure_note.0).unwrap().notes.get_mut(self.current_measure_note.1).unwrap();
        let semitone_searched_note = Semitone::new_from(searched_note);
        println!("searched_note: {:?}", searched_note);
        println!("semitone_searched_note: {:?}", semitone_searched_note);
        
        if &semitone_searched_note == pressed_semitone {
            searched_note.color = Color::GREEN;

            self.current_measure_note.1 += 1;
            if self.current_measure_note.1 == self.staves.get_mut(0).unwrap().measures.get(self.current_measure_note.0).unwrap().notes.len() {
                self.current_measure_note.0 += 1;
                self.current_measure_note.1 = 0;
                
                if self.current_measure_note.0 == self.staves.get_mut(0).unwrap().measures.len() {
                    self.staves.remove(0);
                    self.current_measure_note = (0,0);
                    
                    // TODO: create new stave
                    // measure
                    self.staves.push(Stave::new_random(self.x_pos_stave, self.size_stave, Clef::Sol, Some(KeySignature(KeySignatureAccidental::Sharp, 3))));
                }
            }
            //set the searched note GRAY
            self.staves.get_mut(0).unwrap().measures.get_mut(self.current_measure_note.0).unwrap().notes.get_mut(self.current_measure_note.1).unwrap().color = Color::GRAY;
        }
        
        else{
            searched_note.color = Color::RED;
        }
    }
    
    fn released_semitone(&mut self, released_semitone: &Semitone){
        println!("released_semitone: {:?}", released_semitone);
        println!("self.pressed_semitone: {:?}", self.pressed_semitone);

        if self.pressed_semitone.as_ref() == Some(released_semitone){
            self.pressed_semitone = None;
            
            let searched_note = self.staves.get_mut(0).unwrap().measures.get_mut(self.current_measure_note.0).unwrap().notes.get_mut(self.current_measure_note.1).unwrap();
            searched_note.color = Color::GRAY;
        }
    }

    fn draw(&self, canvas: &WindowCanvas){
        for (i,s) in self.staves.iter().enumerate() {
            s.draw(40+(i*100) as i32, canvas);
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
            let semitone = Semitone(message[1]);
            //0 is key released
            if message[2] == 0 {
                g.lock().unwrap().released_semitone(&semitone);
            }else{
                g.lock().unwrap().pressed_semitone(&semitone);
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