use rand::distributions::Standard;
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;

use sdl2::gfx::primitives::DrawRenderer;

use crate::music::*;

#[derive(Debug, Clone, Copy, Hash)]
struct Note {
    pitch: Pitch,
    accidental: Option<Accidental>,
    octave: Octave,
    color: Color,
    draw_acci: bool,
}

impl Note {
    fn new(pitch: Pitch, accidental: Option<Accidental>, octave: Octave) -> Note {
        Note {
            pitch,
            accidental,
            octave,
            color: Color::BLACK,
            draw_acci: true,
        }
    }
    fn new_random(rng: &mut ThreadRng, c: Clef) -> Note {
        match c {
            Clef::Sol => {
                //A3 -> G5
                Note::new(
                    rng.sample(Standard),
                    rng.sample(Standard),
                    Octave(rng.gen_range(3..=4)),
                )
            }
            Clef::Fa => {
                //A1 -> G3
                Note::new(
                    rng.sample(Standard),
                    rng.sample(Standard),
                    Octave(rng.gen_range(1..=2)),
                )
            }
        }
    }
    fn to_semitone(&self) -> Semitone {
        //A-1 on my piano is 21
        //C0 = 24
        let mut v = 24;
        v += self.octave.0 * 12;
        v += self.pitch.get_semitone_offset() as i32;
        if let Some(a) = self.accidental {
            v += a.get_semitone_offset() as i32;
        }
        Semitone(v as u8)
    }
}

//Note is same independently of its color
impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.to_semitone() == other.to_semitone()
    }
}
impl Eq for Note {}

struct Measure {
    notes: Vec<Note>,
}

impl Measure {
    fn new(mut notes: Vec<Note>, key_sign: KeySignature) -> Measure {
        //we need to treat notes to have coerent accidentals
        let mut previous_accidentals: HashMap<Pitch, Accidental> = HashMap::new();

        for n in notes.iter_mut() {
            if let Some(acci) = previous_accidentals.get(&n.pitch) {
                if n.accidental.is_some() {
                    if n.accidental.unwrap() == *acci {
                        n.draw_acci = false;
                    } else {
                        previous_accidentals.insert(n.pitch, n.accidental.unwrap());
                    }
                }
                //we convert to what was previous
                else {
                    n.accidental = Some(*acci);
                    //but we dont draw accidental
                    n.draw_acci = false;
                }
            } else {
                //first time we encounter this accidental
                if n.accidental.is_some() {
                    if key_sign.is_pitch_inside(n.pitch) {
                        if key_sign.accidental_match(n.accidental.unwrap()) {
                            n.draw_acci = false;
                        } else {
                            previous_accidentals.insert(n.pitch, n.accidental.unwrap());
                        }
                    } else {
                        previous_accidentals.insert(n.pitch, n.accidental.unwrap());
                        if n.accidental == Some(Accidental::Natural) {
                            n.draw_acci = false;
                        }
                    }
                } else if key_sign.is_pitch_inside(n.pitch) {
                    n.accidental = Some(key_sign.get_accidental());
                    n.draw_acci = false;
                }
            }
        }

        Measure { notes }
    }
}

fn get_factor_gap_octave(o: &Octave, clef: &Clef) -> i32 {
    match clef {
        Clef::Sol => (4 - o.0) * 7,
        Clef::Fa => (2 - o.0) * 7,
    }
}

fn get_factor_gap_pitch(p: &Pitch, clef: &Clef) -> i32 {
    let mut r = match p {
        Pitch::A => -5,
        Pitch::B => -6,
        Pitch::C => 0,
        Pitch::D => -1,
        Pitch::E => -2,
        Pitch::F => -3,
        Pitch::G => -4,
    };
    if *clef == Clef::Fa {
        r += 2;
    }
    r + 3
}

struct Stave {
    x_pos: i32,
    size: Point,
    //height between two consecutives notes
    //= radius of notes
    //gap*2 = gap between two lines
    gap: i32,
    clef: Clef,
    key_signature: KeySignature,
    measures: Vec<Measure>,
}

impl Stave {
    #[allow(dead_code)]
    pub fn new(x_pos: i32, size: Point, clef: Clef, key_signature: KeySignature) -> Stave {
        let gap = size.y / 10;

        Stave {
            x_pos,
            size,
            gap,
            measures: Vec::new(),
            key_signature,
            clef,
        }
    }

    pub fn new_random(x_pos: i32, size: Point, clef: Clef, key_signature: KeySignature) -> Stave {
        let mut s = Stave::new(x_pos, size, clef, key_signature);
        let mut rng = rand::thread_rng();
        s.add_measure(Measure::new(
            vec![
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
            ],
            key_signature,
        ));
        s.add_measure(Measure::new(
            vec![
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
            ],
            key_signature,
        ));
        s.add_measure(Measure::new(
            vec![
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
                Note::new_random(&mut rng, clef),
            ],
            key_signature,
        ));
        s
    }

    pub fn add_measure(&mut self, m: Measure) {
        self.measures.push(m);
    }

    pub fn draw(&self, y_pos: i32, canvas: &WindowCanvas) {
        let pos = Point::new(self.x_pos, y_pos);
        //draw lines
        for i in 0..5 {
            canvas
                .thick_line(
                    pos.x as i16,
                    (pos.y + (self.gap * 2 * i) as i32) as i16,
                    (pos.x + self.size.x as i32) as i16,
                    (pos.y + self.gap * 2 * i) as i16,
                    2,
                    Color::BLACK,
                )
                .unwrap();
        }

        //draw clef
        let pos_clef = match self.clef {
            Clef::Sol => pos.y + self.size.y / 2,
            Clef::Fa => pos.y + self.size.y / 2 - 20,
        };
        canvas
            .string(
                pos.x as i16 - 23,
                pos_clef as i16,
                &self.clef.to_string(),
                Color::BLACK,
            )
            .unwrap();

        //draw key_signature
        let small_gap_x = self.size.x / 60;
        let s;
        let order;
        match self.key_signature.0 {
            KeySignatureAccidental::Sharp => {
                s = '#';
                order = ORDER_SIGNATURE_SHARP;
            }
            KeySignatureAccidental::Flat => {
                s = 'b';
                order = ORDER_SIGNATURE_FLAT;
            }
        }

        for i in 0..self.key_signature.get_number() {
            let y = pos.y + get_factor_gap_pitch(&order[i as usize], &self.clef) * self.gap + 1
                - self.gap;
            let x = pos.x + small_gap_x * i as i32;
            canvas
                .character(x as i16, y as i16, s, Color::BLACK)
                .unwrap();
        }

        let mut current_x = pos.x + small_gap_x * 8;
        let gap_x = self.size.x / 15;
        //draw measures
        for m in self.measures.iter() {
            //draw measures separating lines
            canvas
                .thick_line(
                    current_x as i16,
                    pos.y as i16,
                    current_x as i16,
                    (pos.y + self.gap * 8) as i16,
                    2,
                    Color::BLACK,
                )
                .unwrap();
            current_x += gap_x;


            //draw notes
            for n in m.notes.iter() {
                let nb_factor_gap = get_factor_gap_pitch(&n.pitch, &self.clef)
                    + get_factor_gap_octave(&n.octave, &self.clef);
                let y = pos.y + nb_factor_gap * self.gap;
                let x = current_x;
                canvas
                    .filled_circle(x as i16, y as i16, self.gap as i16, n.color)
                    .unwrap();

                //draw accidental
                if n.draw_acci && n.accidental.is_some() {
                    match n.accidental.unwrap() {
                        Accidental::Sharp => canvas
                            .character((x - small_gap_x - 2) as i16, (y - 4) as i16, '#', n.color)
                            .unwrap(),
                        Accidental::Flat => canvas
                            .character((x - small_gap_x - 2) as i16, (y - 4) as i16, 'b', n.color)
                            .unwrap(),
                        Accidental::Natural => canvas
                            .character((x - small_gap_x - 2) as i16, (y - 4) as i16, 'n', n.color)
                            .unwrap(),
                    }
                }

                //draw help lines
                let help_line_width = (self.gap as f32 * 1.5) as i32;
                if nb_factor_gap <= -2 {
                    for i_y in (2..=-nb_factor_gap).step_by(2) {
                        let y = pos.y + i_y * -self.gap;
                        canvas
                            .thick_line(
                                (x - help_line_width) as i16,
                                y as i16,
                                (x + help_line_width) as i16,
                                y as i16,
                                2,
                                n.color,
                            )
                            .unwrap();
                    }
                } else if nb_factor_gap >= 10 {
                    for i_y in (10..=nb_factor_gap).step_by(2) {
                        let y = pos.y + i_y * self.gap;
                        canvas
                            .thick_line(
                                (x - help_line_width) as i16,
                                y as i16,
                                (x + help_line_width) as i16,
                                y as i16,
                                2,
                                n.color,
                            )
                            .unwrap();
                    }
                }

                current_x += gap_x;
            }
        }
    }
}

pub struct Game {
    size_stave: Point,
    x_pos_stave: i32,
    staves: Vec<Stave>,
    current_measure_note: (usize, usize),
    pressed_semitone: Option<Semitone>,
    score: (u32, u32)
}

impl Game {
    pub fn new(screen_width: u32) -> Game {
        let width = (screen_width as f32 - (screen_width as f32 * 0.1)) as i32;
        let height = 50;
        let size_stave = Point::new(width, height);
        let x_pos_stave = ((screen_width as f32 - width as f32) / 2. as f32) as i32;

        let mut staves = Vec::new();
        for _ in 0..4 {
            staves.push(Stave::new_random(
                x_pos_stave,
                size_stave,
                rand::random(),
                rand::random(),
            ));
        }

        let current_measure_note = (0, 0);
        staves
            .get_mut(0)
            .unwrap()
            .measures
            .get_mut(current_measure_note.0)
            .unwrap()
            .notes
            .get_mut(current_measure_note.1)
            .unwrap()
            .color = Color::GRAY;

        Game {
            size_stave,
            x_pos_stave,
            staves,
            current_measure_note,
            pressed_semitone: None,
            score: (0,0),
        }
    }

    pub fn parse_midi_message(&mut self, message: &[u8]) {
        // println!("{:?}", message);
        let semitone = Semitone(message[1]);
        //0 is key released
        if message[2] == 0 {
            self.released_semitone(&semitone);
        } else {
            self.pressed_semitone(&semitone);
        }
    }

    fn pressed_semitone(&mut self, pressed_semitone: &Semitone) {
        println!("pressed_semitone: {:?}", pressed_semitone);
        self.pressed_semitone = Some(*pressed_semitone);

        let searched_note = self
            .staves
            .get_mut(0)
            .unwrap()
            .measures
            .get_mut(self.current_measure_note.0)
            .unwrap()
            .notes
            .get_mut(self.current_measure_note.1)
            .unwrap();
        let semitone_searched_note = searched_note.to_semitone();
        println!("searched_note: {:?}", searched_note);
        println!("semitone_searched_note: {:?}", semitone_searched_note);

        if &semitone_searched_note == pressed_semitone {
            self.score.0 += 1;

            searched_note.color = Color::GREEN;

            self.current_measure_note.1 += 1;
            if self.current_measure_note.1
                == self
                    .staves
                    .get_mut(0)
                    .unwrap()
                    .measures
                    .get(self.current_measure_note.0)
                    .unwrap()
                    .notes
                    .len()
            {
                self.current_measure_note.0 += 1;
                self.current_measure_note.1 = 0;

                if self.current_measure_note.0 == self.staves.get_mut(0).unwrap().measures.len() {
                    self.staves.remove(0);
                    self.current_measure_note = (0, 0);

                    // TODO: create new stave
                    // measure
                    self.staves.push(Stave::new_random(
                        self.x_pos_stave,
                        self.size_stave,
                        Clef::Sol,
                        KeySignature::new(KeySignatureAccidental::Sharp, 0),
                    ));
                }
            }
            //set the searched note GRAY
            self.staves
                .get_mut(0)
                .unwrap()
                .measures
                .get_mut(self.current_measure_note.0)
                .unwrap()
                .notes
                .get_mut(self.current_measure_note.1)
                .unwrap()
                .color = Color::GRAY;
        } else {
            searched_note.color = Color::RED;
        }
        self.score.1 += 1;
    }

    fn released_semitone(&mut self, released_semitone: &Semitone) {
        println!("released_semitone: {:?}", released_semitone);
        println!("self.pressed_semitone: {:?}", self.pressed_semitone);

        if self.pressed_semitone.as_ref() == Some(released_semitone) {
            self.pressed_semitone = None;

            let searched_note = self
                .staves
                .get_mut(0)
                .unwrap()
                .measures
                .get_mut(self.current_measure_note.0)
                .unwrap()
                .notes
                .get_mut(self.current_measure_note.1)
                .unwrap();
            searched_note.color = Color::GRAY;
        }
    }

    pub fn draw(&self, canvas: &WindowCanvas) {
        for (i, s) in self.staves.iter().enumerate() {
            s.draw(40 + (i * 150) as i32, canvas);
        }

        canvas.string(5,5, &((self.score.0).to_string()+"/"+&(self.score.1).to_string()), Color::BLACK).unwrap();
    }
}
