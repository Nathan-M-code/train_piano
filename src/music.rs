use std::fmt;

use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

//A-1 on my piano is 21
//C0 = 24
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Semitone(pub u8);

//from -1 to 7 on my piano
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Octave(pub i32);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Pitch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Accidental {
    Sharp,
    Flat,
    Natural,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Clef {
    Sol,
    Fa,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KeySignatureAccidental {
    Sharp,
    Flat,
}

#[derive(Debug, Clone, Copy)]
pub struct KeySignature(pub KeySignatureAccidental, u8);

pub const ORDER_SIGNATURE_SHARP: [Pitch; 7] = [
    Pitch::F,
    Pitch::C,
    Pitch::G,
    Pitch::D,
    Pitch::A,
    Pitch::E,
    Pitch::B,
];
pub const ORDER_SIGNATURE_FLAT: [Pitch; 7] = [
    Pitch::B,
    Pitch::E,
    Pitch::A,
    Pitch::D,
    Pitch::G,
    Pitch::C,
    Pitch::F,
];

impl Accidental {
    pub fn get_semitone_offset(&self) -> i8 {
        match self {
            Self::Sharp => 1,
            Self::Flat => -1,
            Self::Natural => 0,
        }
    }
}

impl KeySignature {
    pub fn new(accidental: KeySignatureAccidental, nb: u8) -> KeySignature {
        let nb = nb.clamp(0, 7);
        KeySignature(accidental, nb)
    }

    pub fn get_number(&self) -> u8 {
        self.1
    }

    pub fn is_pitch_inside(&self, p: Pitch) -> bool {
        match self.0 {
            KeySignatureAccidental::Sharp => {
                for i in 0..self.1 {
                    if ORDER_SIGNATURE_SHARP[i as usize] == p {
                        return true;
                    }
                }
                return false;
            }
            KeySignatureAccidental::Flat => {
                for i in 0..self.1 {
                    if ORDER_SIGNATURE_FLAT[i as usize] == p {
                        return true;
                    }
                }
                return false;
            }
        }
    }

    pub fn accidental_match(&self, a: Accidental) -> bool {
        if self.0 == KeySignatureAccidental::Sharp && a == Accidental::Sharp {
            return true;
        }
        if self.0 == KeySignatureAccidental::Flat && a == Accidental::Flat {
            return true;
        }
        false
    }

    pub fn get_accidental(&self) -> Accidental {
        match self.0 {
            KeySignatureAccidental::Sharp => Accidental::Sharp,
            KeySignatureAccidental::Flat => Accidental::Flat,
        }
    }
}

impl Pitch {
    pub fn get_semitone_offset(&self) -> u8 {
        match self {
            Self::A => 9,
            Self::B => 11,
            Self::C => 0,
            Self::D => 2,
            Self::E => 4,
            Self::F => 5,
            Self::G => 7,
        }
    }
}

impl fmt::Display for Clef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
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

impl Distribution<Clef> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Clef {
        match rng.gen_bool(0.5) {
            true => Clef::Sol,
            false => Clef::Fa,
        }
    }
}

impl Distribution<KeySignatureAccidental> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> KeySignatureAccidental {
        match rng.gen_bool(0.5) {
            true => KeySignatureAccidental::Sharp,
            false => KeySignatureAccidental::Flat,
        }
    }
}

impl Distribution<KeySignature> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> KeySignature {
        let nb = match rng.gen_bool(0.2) {
            true => rng.gen_range(1..6),
            false => 0
        };
        KeySignature::new(rng.sample(Standard), nb)
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
            _ => Pitch::G,
        }
    }
}
