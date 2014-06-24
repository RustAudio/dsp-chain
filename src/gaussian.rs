
use std::rand::random;
use std::num;
use std::cell::Cell;
use math;

/// A type for generating values pseudo-
/// randomly from a gaussian distribution.
#[deriving(Clone, Show)]
pub struct Gaussian {
    /// States whether there is a pre-prepared
    /// value to use.
    pub have_next_value: Cell<bool>,
    /// A holder for a pre-prepared value.
    pub next_value: Cell<f32>
}

impl Gaussian {

    /// Constructor.
    pub fn new() -> Gaussian {
        Gaussian {
            have_next_value: Cell::new(false),
            next_value: Cell::new(0f32)
        }
    }

    /// Generate raw gaussian value with mean of 0.
    pub fn gen_raw(&self) -> f32 {
        if self.have_next_value.get() {
            self.have_next_value.set(false);
            return self.next_value.get();
        }
        else {
            let mut va @ mut vb @ mut s = 0.0f32;
            while s >= 1.0f32 || s == 0.0f32 {
                va = 2.0f32 * random::<f32>() - 1.0f32;
                vb = 2.0f32 * random::<f32>() - 1.0f32;
                s = va * vb + va * vb
            };
            let multi: f32 = ((-2.0) * s.abs().ln() / s).abs().sqrt();
            self.next_value.set(vb * multi);
            self.have_next_value.set(true);
            return va * multi;
        }
    }

    /// Generate gaussian value with mean of `mean` with `rand` randomness.
    pub fn gen(&self, mean: f32, rand: f32) -> f32 {
        let mut ans = self.gen_raw() * num::pow(rand, 2) + (mean * 2.0 - 1.0);
        ans = jmath::map(ans, -1.0, 1.0, 0.0, 1.0);
        if ans > 1.0 || ans < 0.0 {
            return self.gen(mean, rand);
        }
        return ans;
    }

    /// Generate gaussian value mapped to a range.
    pub fn gen_map(&self, mean: f32, rand: f32, min_range: f32, max_range: f32) -> f32 {
        let perc: f32 = (mean - min_range) / (max_range - min_range);
        self.gen(perc, rand) * (max_range - min_range) + min_range
    }

}

#[test]
pub fn test() {

    let g = Gaussian::new();

    // Test 'gen_raw'
    print!("Raw Gaussian = ");
    for _ in range(0,10) {
        print!("{}, ", g.gen_raw());
    }
    println!("");

    // Test 'Gaussian.gen'
    print!("Gaussian .5 with rand .1 = ");
    for _ in range(0,10) {
        print!("{}, ", g.gen(0.5, 0.1));
    }
    println!("");

}
*/
//------------------------------
