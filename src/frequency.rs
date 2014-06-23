

use std::num::abs;
use self::calculator::*;
use self::constants::*;

/// Frequency related constants.
pub mod constants {
    pub static HIGHEST_HZ: f32 = 20000f32;
    pub static LOWEST_HZ: f32 = 20f32;
    pub static DEFAULT_WEIGHT: f32 = 4f32;
}

/// Frequency struct! Capable of scaling
/// frequency weight via a power function
/// applied to a percentage.
#[deriving(Show, Clone)]
pub struct Frequency {
    hz: f32,
    perc: f64,
    scaled_perc: f64, // For weighting lower frequencies.
    scale_weight: f32 // For adjusting the scale weighting (default 4).
}

/// Frequency calculator for converting between
/// scale percentage and frequency. in all
/// possible directions.
pub mod calculator {

    use super::constants::*;

    pub fn get_hz_from_perc(perc: f64) -> f32 {
        perc as f32 * (HIGHEST_HZ - LOWEST_HZ) + LOWEST_HZ
    }

    pub fn get_hz_from_scaled_perc(scaled: f64, weight: f32) -> f32 {
        get_hz_from_perc(get_perc_from_scaled_perc(scaled, weight))
    }

    pub fn get_perc_from_hz(hz: f32) -> f64 {
        (hz - LOWEST_HZ) as f64 / (HIGHEST_HZ - LOWEST_HZ) as f64
    }

    pub fn get_perc_from_scaled_perc(scaled: f64, weight: f32) -> f64 {
        scaled.powf(weight as f64)
    }

    pub fn get_scaled_perc_from_hz(hz: f32, weight: f32) -> f64 {
        get_scaled_perc_from_perc(get_perc_from_hz(hz), weight)
    }

    pub fn get_scaled_perc_from_perc(perc: f64, weight: f32) -> f64 {
        perc.powf(1f64 / weight as f64)
    }

}

/// A HasFrequency for easy implementation
/// for those types that revolve around frequency.
/// This is also the base trait for the more
/// musically orientated `HasPitch` trait.
pub trait HasFrequency {

    /// Getters
    fn get_freq<'a>(&'a self) -> &'a Frequency;
    fn get_freq_mut<'a>(&'a mut self) -> &'a mut Frequency;
    fn get_hz(&self) -> f32 { self.get_freq().hz }
    fn get_perc(&self) -> f64 { self.get_freq().perc }
    fn get_scaled_perc(&self) -> f64 { self.get_freq().scaled_perc }
    fn get_scale_weight(&self) -> f32 { self.get_freq().scale_weight }

    /// Setters
    fn set_freq_hz(&mut self, hz: f32) {
        assert!(hz >= LOWEST_HZ && hz <= HIGHEST_HZ);
        let weight = self.get_freq().scale_weight;
        self.get_freq_mut().hz = hz;
        self.get_freq_mut().perc = get_perc_from_hz(hz);
        self.get_freq_mut().scaled_perc = get_scaled_perc_from_hz(hz, weight);
    }
    fn set_freq_perc(&mut self, perc: f64) {
        assert!(perc >= 0f64 && perc <= 1f64);
        let weight = self.get_freq().scale_weight;
        self.get_freq_mut().hz = get_hz_from_perc(perc);
        self.get_freq_mut().perc = perc;
        self.get_freq_mut().scaled_perc = get_scaled_perc_from_perc(perc, weight);
    }
    fn set_freq_scaled_perc(&mut self, scaled_perc: f64) {
        assert!(scaled_perc >= 0f64 && scaled_perc <= 1f64);
        let weight = self.get_freq().scale_weight;
        self.get_freq_mut().hz = get_hz_from_scaled_perc(scaled_perc, weight);
        self.get_freq_mut().perc = get_perc_from_scaled_perc(scaled_perc, weight);
        self.get_freq_mut().scaled_perc = scaled_perc;
    }
    fn set_scale_weight(&mut self, scale_weight: f32) {
        self.get_freq_mut().scale_weight = scale_weight;
    }

    /// Determine difference as another Frequency object.
    fn find_difference(&self, freq: &Frequency) -> Frequency {
        Frequency::new_from_scaled_perc_and_weight(abs(self.get_scaled_perc() - freq.get_scaled_perc()), self.get_scale_weight())
    }
    
}

/// Only methods returning a reference to
/// Frequency both mutably and immutably
/// are required to impl HasFrequency.
impl HasFrequency for Frequency {
    fn get_freq<'a>(&'a self) -> &'a Frequency { self }
    fn get_freq_mut<'a>(&'a mut self) -> &'a mut Frequency { self }
}


impl Frequency {

    /// Default constructor.
    pub fn new() -> Frequency {
        Frequency {
            hz: LOWEST_HZ,
            perc: 0f64,
            scaled_perc: 0f64,
            scale_weight: DEFAULT_WEIGHT
        }
    }

    /// Construct from hz.
    pub fn new_from_hz(hz: f32) -> Frequency {
        Frequency {
            hz: hz,
            perc: get_perc_from_hz(hz),
            scaled_perc: get_scaled_perc_from_hz(hz, DEFAULT_WEIGHT),
            scale_weight: DEFAULT_WEIGHT
        }
    }

    /// Construct from hz percentage.
    pub fn new_from_perc(perc: f64) -> Frequency {
        Frequency {
            hz: get_hz_from_perc(perc),
            perc: perc,
            scaled_perc: get_scaled_perc_from_perc(perc, DEFAULT_WEIGHT),
            scale_weight: DEFAULT_WEIGHT
        }
    }

    /// Construct from scaled percentage. (Weighted
    /// towards the lowever end of the spectrum).
    pub fn new_from_scaled_perc(scaled_perc: f64) -> Frequency {
        Frequency {
            hz: get_hz_from_scaled_perc(scaled_perc, DEFAULT_WEIGHT),
            perc: get_perc_from_scaled_perc(scaled_perc, DEFAULT_WEIGHT),
            scaled_perc: scaled_perc,
            scale_weight: DEFAULT_WEIGHT
        }
    }

    /// Construct from scaled percentage. (Weighted
    /// towards the lowever end of the spectrum).
    /// The low-end weight can be defined here.
    pub fn new_from_scaled_perc_and_weight(scaled_perc: f64, weight: f32) -> Frequency {
        Frequency {
            hz: get_hz_from_scaled_perc(scaled_perc, weight),
            perc: get_perc_from_scaled_perc(scaled_perc, weight),
            scaled_perc: scaled_perc,
            scale_weight: weight
        }
    }

}

// Tests
//------------------------------
/*
pub fn test() {

    println!("Frequency Tests!");
    let mut freq = Frequency::new();
    println!("Initialised freq: {}", freq);

    freq.set_freq_hz(1000f32);
    println!("freq set to 1,000 hz: {}", freq);

    freq.set_freq_perc(0.8f64);
    println!("freq set to 0.8 perc: {}", freq);

    freq.set_freq_scaled_perc(0.5f64);
    println!("freq set to 0.5 scaled perc: {}", freq);

    freq = Frequency::new_from_scaled_perc_and_weight(0.5f64, 2f32);
    println!("freq initialised to 0.5 scaled perc with 2.0 weight: {}", freq);

    freq = Frequency::new_from_hz(440f32);
    println!("freq initialised to 440 hz: {}", freq);

    freq = Frequency::new_from_perc(0.5f64);
    println!("freq initialised to 0.5 perc: {}", freq);

    freq = Frequency::new_from_scaled_perc(0.5f64);
    println!("freq initialised to 0.5 scaled perc: {}", freq);

}
*/
//------------------------------
