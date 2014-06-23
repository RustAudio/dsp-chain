
use frequency::{Frequency, HasFrequency};
use frequency::calculator::*;
use frequency::constants::*;
use self::letter::*;
use std::num::abs;
use std::fmt::{Formatter, Result};
use std::fmt;

/// Useful for Pitch calculations.
static TWELFTH_ROOT_OF_TWO: f32 = 1.059463094359f32;
/// The pitch `A 4` represented in steps.
static TUNING_PITCH_A4: f32 = 57f32;
/// The pitch `A 4` represented in hz.
static PITCH_INDEX: f32 = 440f32;


pub mod letter {

    /// Represent the scale letter
    /// as a u8 type.
    pub type Letter = u8;
    pub static C: Letter = 0;
    pub static Csh: Letter = 1;
    pub static Db: Letter = Csh;
    pub static D: Letter = 2;
    pub static Dsh: Letter = 3;
    pub static Eb: Letter = Dsh;
    pub static E: Letter = 4;
    pub static F: Letter = 5;
    pub static Fsh: Letter = 6;
    pub static Gb: Letter = Fsh;
    pub static G: Letter = 7;
    pub static Gsh: Letter = 8;
    pub static Ab: Letter = Gsh;
    pub static A: Letter = 9;
    pub static Ash: Letter = 10;
    pub static Bb: Letter = Ash;
    pub static B: Letter = 11;
    pub static TOTAL_LETTERS: Letter = 12;

    /// Get as string for printing / 
    /// GUI display / debugging.
    pub fn get_str_from_letter(letter: Letter) -> &str {
        match letter {
            0  => "C",
            1  => "Csh",
            2  => "D",
            3  => "Dsh",
            4  => "E",
            5  => "F",
            6  => "Fsh",
            7  => "G",
            8  => "Gsh",
            9  => "A",
            10 => "Ash",
            11 => "B",
            _ => "Letter out of range! Must be between 0 - 11 u8."
        }
    }

}

/// Represents musical pitch as a note
/// letter (u8) and octave (int).
#[deriving(Clone)]
pub struct LetterOctave (Letter, int);

impl fmt::Show for LetterOctave {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.get_str())
    }
}

impl LetterOctave {
    /// Constructor for LetterOctave representation
    /// of pitch.
    pub fn new(letter: Letter, octave: int) -> LetterOctave {
        LetterOctave(letter, octave)
    }
    /// Returns the letter from the tuple.
    pub fn letter(&self) -> Letter {
        match *self { LetterOctave(letter, _) => letter }
    }
    /// Returns the octave from the tuple.
    pub fn octave(&self) -> int {
        match *self { LetterOctave(_, octave) => octave }
    }
    /// Return as string for Show implementation.
    pub fn get_str(&self) -> String {
        format!("LetterOctave: {}, {}", get_str_from_letter(self.letter()), self.octave().to_str())
    }
}


/// Pitch object built to handle a variety of musical pitch representations.
#[deriving(Show, Clone)]
pub struct Pitch {
    /// Frequency representation of musical pitch.
    frequency: Frequency,
    /// Represents pitch as a floating point step (akin to MIDI's 0-128).
    step: f32
}


/// Calculate pitch as LetterOctave from pitch as step.
pub fn get_letter_octave_from_step(step: f32) -> LetterOctave {
    let rounded: int = step.round() as int;
    let letter_step: int = rounded % TOTAL_LETTERS as int;
    LetterOctave(letter_step as u8, (rounded - letter_step) / 12)
}

/// Calculate LetterOctave from hz.
pub fn get_letter_octave_from_hz(hz: f32, pitch_index: f32) -> LetterOctave {
    get_letter_octave_from_step(get_step_from_hz(hz, pitch_index))
}

/// Calculate LetterOctave from a frequency percentage.
pub fn get_letter_octave_from_perc(perc: f64, pitch_index: f32) -> LetterOctave {
    get_letter_octave_from_step(get_step_from_perc(perc, pitch_index))
}

/// Calculate LetterOctave from a scaled frequency percentage.
pub fn get_letter_octave_from_scaled_perc(scaled: f64, weight: f32,
                                          pitch_index: f32) -> LetterOctave {
    get_letter_octave_from_step(get_step_from_scaled_perc(scaled, weight, pitch_index))
}

/// Calculate the pitch `step` from LetterOctave.
pub fn get_step_from_letter_octave(letter_octave: LetterOctave) -> f32 {
    let (letter, octave) = match letter_octave {
        LetterOctave(letter, octave) => (letter, octave)
    };
    octave as f32 * 12f32 + letter as f32
}

/// Calculate the pitch `step` from frequency in hz.
pub fn get_step_from_hz(hz: f32, pitch_index: f32) -> f32 {
    (hz / pitch_index).log2() / TWELFTH_ROOT_OF_TWO.log2() + TUNING_PITCH_A4
}

/// Calculate the pitch `step` from frequency precentage.
pub fn get_step_from_perc(perc: f64, pitch_index: f32) -> f32 {
    get_step_from_hz(get_hz_from_perc(perc), pitch_index)
}

/// Calculate the pitch `step` from a scaled frequency precentage.
pub fn get_step_from_scaled_perc(scaled: f64, weight: f32, pitch_index: f32) -> f32 {
    get_step_from_hz(get_hz_from_scaled_perc(scaled, weight), pitch_index)
}

/// Calculate hz from LetterOctave.
pub fn get_hz_from_letter_octave(letter_octave: LetterOctave, pitch_index: f32) -> f32 {
    get_hz_from_step(get_step_from_letter_octave(letter_octave), pitch_index)
}

/// Calculate hz from pitch as `step`.
pub fn get_hz_from_step(step: f32, pitch_index: f32) -> f32 {
    pitch_index * TWELFTH_ROOT_OF_TWO.powf(step - TUNING_PITCH_A4)
}

/// Calculate frequency percentage from pitch as `step`.
pub fn get_perc_from_step(step: f32, pitch_index: f32) -> f64 {
    get_perc_from_hz(get_hz_from_step(step, pitch_index))
}

/// Calculate scaled frequency percentage from pitch as `step`.
pub fn get_scaled_perc_from_step(step: f32, weight: f32, pitch_index: f32) -> f64 {
    get_scaled_perc_from_hz(get_hz_from_step(step, pitch_index), weight)
}

/// Find and return the smallest distance
/// between two letters in semitones as an int.
pub fn get_difference_in_semitones(letter_a: Letter, letter_b: Letter) -> int {
    let diff = abs(letter_a as int - letter_b as int);
    match diff > 6 {
        true => diff - 12,
        false => diff
    }
}

/// Print out a table of letters, octaves,
/// frequency and step within the specified
/// octave range.
pub fn print_note_freq_table(oct_a: int, oct_b: int, pitch_index: f32) {
    for i in range(oct_a, oct_b) {
        for j in range (0, TOTAL_LETTERS) {
            println!("{} | hz = {} | step = {}", LetterOctave(j, i),
            get_hz_from_letter_octave(LetterOctave(j, i), pitch_index), 
            get_step_from_letter_octave(LetterOctave(j, i)));
        }
    }
}

/// A trait for representing types that
/// revolved around the use of Pitch.
/// HasPitch expands upon the methodology
/// of HasFrequency.
pub trait HasPitch: HasFrequency {

    /// Return an immutable reference to the Pitch struct.
    fn get_pitch<'a>(&'a self) -> &'a Pitch;
    /// Return a mutable reference to the Pitch struct.
    fn get_pitch_mut<'a>(&'a mut self) -> &'a mut Pitch;
    /// Return pitch in the form of a MIDI-esque step (0 - 128).
    fn get_step(&self) -> f32 { self.get_pitch().step }
    /// Return the pitch index (A 4 represented in hz)
    fn get_pitch_index(&self) -> f32 { PITCH_INDEX }
    /// Return pitch in the form of a note `Letter` and octave (int).
    fn get_letter_octave(&self) -> LetterOctave {
        get_letter_octave_from_step(self.get_step())
    }

    /// Setters (set all values from given pitch representation).
    fn set_letter_octave(&mut self, letter_octave: LetterOctave) {
        self.get_pitch_mut().step = get_step_from_letter_octave(letter_octave);
        let hz = get_hz_from_letter_octave(letter_octave, PITCH_INDEX);
        self.set_freq_hz(hz)
    }
    /// Set pitch via `step` (0 - 128).
    fn set_step(&mut self, step: f32) {
        self.get_pitch_mut().step = step;
        let hz = get_hz_from_step(step, PITCH_INDEX);
        self.set_freq_hz(hz);
    }
    /// Set pitch via frequency in hz.
    fn set_hz(&mut self, hz: f32) {
        self.get_pitch_mut().step = get_step_from_hz(hz, PITCH_INDEX);
        self.set_freq_hz(hz);
    }
    /// Set pitch via frequency precentage.
    fn set_perc(&mut self, perc: f64) {
        self.get_pitch_mut().step = get_step_from_perc(perc, PITCH_INDEX);
        self.set_freq_perc(perc);
    }
    /// Set pitch via scaled frequency precentage.
    fn set_scaled_perc(&mut self, scaled_perc: f64) {
        let weight = self.get_scale_weight();
        self.get_pitch_mut().step =
            get_step_from_scaled_perc(scaled_perc, weight, PITCH_INDEX);
        self.set_freq_scaled_perc(scaled_perc);
    }

}


impl HasPitch for Pitch {    
    /// Return an immutable reference to the Pitch struct.
    fn get_pitch<'a>(&'a self) -> &'a Pitch { self }
    /// Return a mutable reference to the Pitch struct.
    fn get_pitch_mut<'a>(&'a mut self) -> &'a mut Pitch { self }
}

impl HasFrequency for Pitch {
    /// Get Frequency struct as an immutable reference.
    fn get_freq<'a>(&'a self) -> &'a Frequency { &self.frequency }
    /// Get Frequency struct as a mutable reference.
    fn get_freq_mut<'a>(&'a mut self) -> &'a mut Frequency { &mut self.frequency }
}


impl Pitch {

    /// Create a new 0 initialised Pitch object.
    pub fn new() -> Pitch {
        Pitch {
            frequency: Frequency::new(),
            step: get_step_from_hz(LOWEST_HZ, PITCH_INDEX) // equivalent of 0 perc / 20hz.
        }
    }

    /// Create a new Pitch object initialised by a step (i.e. between 0, 128).
    pub fn new_from_step(step: f32) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_hz(get_hz_from_step(step, PITCH_INDEX)),
            step: step
        }
    }

    /// Create a new Pitch object initialised by letter and octave (i.e. Ash, 4).
    pub fn new_from_letter_octave(letter_octave: LetterOctave) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_hz(get_hz_from_letter_octave(letter_octave, PITCH_INDEX)),
            step: get_step_from_letter_octave(letter_octave)
        }
    }
    
    /// Create a new Pitch object initialised by frequency in hz.
    pub fn new_from_hz(hz: f32) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_hz(hz),
            step: get_step_from_hz(hz, PITCH_INDEX)
        }
    }

    /// Create a new Pitch object initialised by a percentage of the frequency range (hz).
    pub fn new_from_perc(perc: f64) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_perc(perc),
            step: get_step_from_perc(perc, PITCH_INDEX)
        }
    }

    /// Create a new Pitch object initialised by a scaled percentage.
    pub fn new_from_scaled_perc(scaled_perc: f64) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_scaled_perc(scaled_perc),
            step: get_step_from_scaled_perc(scaled_perc, DEFAULT_WEIGHT, PITCH_INDEX)
        }
    }

    /// Create a new Pitch object initialised by a scaled percentage and specific weighting.
    pub fn new_from_scaled_perc_and_weight(scaled_perc: f64, weight: f32) -> Pitch {
        Pitch {
            frequency: Frequency::new_from_scaled_perc_and_weight(scaled_perc, weight),
            step: get_step_from_scaled_perc(scaled_perc, weight, PITCH_INDEX)
        }
    }

}

// Tests
//------------------------------

/*
pub fn test() {

    println!("Pitch Tests!");

    // Calculator
    println!("Get Letter and Octave from step (57): {}", get_letter_octave_from_step(57f32));
    println!("Get Letter and Octave from hz (440): {}", get_letter_octave_from_hz(440f32, PITCH_INDEX));
    println!("Get Letter and Octave from perc (0.5): {}", get_letter_octave_from_perc(0.5f64, PITCH_INDEX));
    println!("Get Letter and Octave from scaled_perc (0.5): {}", get_letter_octave_from_scaled_perc(0.5f64, DEFAULT_WEIGHT, PITCH_INDEX));
    println!("Get Step from Letter and Octave (C, 2): {}", get_step_from_letter_octave(LetterOctave(C, 2)));
    println!("Get Step from hz (440): {}", get_step_from_hz(440f32, PITCH_INDEX));
    println!("Get Step from perc (0.3): {}", get_step_from_perc(0.3f64, PITCH_INDEX));
    println!("Get Step from scaled_perc (0.3): {}", get_step_from_scaled_perc(0.3f64, DEFAULT_WEIGHT, PITCH_INDEX));
    println!("Get hz from letter & octave (A, 4): {}", get_hz_from_letter_octave(LetterOctave(A, 4), PITCH_INDEX));
    println!("Get hz from step (57): {}", get_hz_from_step(57f32, PITCH_INDEX));
    println!("Get perc from step (440): {}", get_perc_from_step(57f32, PITCH_INDEX));
    println!("Get scaled_perc from step (440): {}", get_scaled_perc_from_step(57f32, DEFAULT_WEIGHT, PITCH_INDEX));

    let mut pitch = Pitch::new();
    println!("Initialised Pitch: {}:", pitch);

    // Setters
    pitch.set_step(57f32);
    println!("Pitch with step set to 57: {}:", pitch);
    pitch.set_letter_octave(LetterOctave(C, 2));
    println!("Pitch with letter octave set to C 2: {}:", pitch);
    pitch.set_hz(440f32);
    println!("Pitch with hz set to 440: {}:", pitch);
    pitch.set_perc(0.25f64);
    println!("Pitch with perc 0.25: {}:", pitch);
    pitch.set_scaled_perc(0.25f64);
    println!("Pitch with perc 0.25: {}:", pitch);

    // Constructors
    pitch = Pitch::new_from_step(48f32);
    println!("Pitch with step set to 48: {}:", pitch);
    pitch = Pitch::new_from_letter_octave(LetterOctave(A, 3));
    println!("Pitch with letter octave set to A 3: {}:", pitch);
    pitch = Pitch::new_from_hz(880f32);
    println!("Pitch with hz set to 880: {}:", pitch);
    pitch = Pitch::new_from_perc(0.1f64);
    println!("Pitch with perc set to 0.1: {}:", pitch);
    pitch = Pitch::new_from_scaled_perc(0.1f64);
    println!("Pitch with scaled perc set to 0.1: {}:", pitch);
    pitch = Pitch::new_from_scaled_perc_and_weight(0.1f64, 3f32);
    println!("Pitch with scaled perc set to 0.1 and weight of 3: {}:", pitch);

    // Semitones difference
    println!("The difference between Csh and Gsh in semitones is {}", get_difference_in_semitones(Csh, Gsh));
    
    // Print Note Table
    print_note_freq_table(4, 5, PITCH_INDEX);

}
*/

//------------------------------
