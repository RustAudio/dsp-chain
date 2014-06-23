
use std::num::{One, Zero};
use std::fmt;
use std::rand;
use math;


#[deriving(Clone, Show)]
/// Signal generic struct for simplifying dsp signal generation.
pub struct Signal<F> {
    pub val: F,
    x: F,
    y: F,
    grad: F,
    pub bez_depth: F,
    pub freq: F,
    pub amp: F
}


/// Times two pi for most methods in 'Signal' implementations.
fn times_two_pi<F: FloatMath>(f: F) -> F { f * Float::two_pi() }

/// Get random() mapped from -1.0 to 1.0 for 'Signal::get_noise'.
fn get_rand_signal<F: FloatMath + rand::Rand + FromPrimitive>() -> F {
    let r: F = rand::random();
    r * FromPrimitive::from_f32(2.0f32).unwrap() - FromPrimitive::from_f32(1.0f32).unwrap()
}


impl<F: FloatMath + rand::Rand + FromPrimitive + ToPrimitive + Signed> Signal<F> {

    /// Constructor for Signal
    pub fn new(val: F) -> Signal<F> {
        Signal {
            val: val,
            x: One::one(),
            y: Zero::zero(),
            grad: Zero::zero(),
            bez_depth: Zero::zero(),
            freq: One::one(),
            amp: One::one()
        }
    }

    /// Set value for which you will return signal (get_sin/cos/sqr/saw) etc...
    pub fn set_val(&mut self, val: F) {
        self.val = val;
    }

    /// If you woudl like to return the signal value on a slope, set gradient here.
    pub fn set_gradient(&mut self, x: F, y: F) {
        self.x = x;
        self.y = y;
        self.grad = x / y;
    }

    /// Set frequency of signal.
    pub fn set_freq(&mut self, freq: F) {
        self.freq = freq;
    }

    /// Set amplitude of signal.
    pub fn set_amp(&mut self, amp: F) {
        self.amp = amp;
    }

    /// Set depth of bezier curve. Defaults to 0.
    pub fn set_bez_depth(&mut self, bez_depth: F) {
        self.bez_depth = bez_depth;
    }

    /// Helper function for 'get_bezier'.
    fn get_bezier_pt(na: F, nb: F, perc: F) -> F {
        let diff: F = nb - na;
        diff * perc + na
    }

    /// Helper function for 'get_bezier'.
    fn get_y1(y: F, one: F) -> F {
        y/(one+one)
    }

    /// Get signal with bezier curve.
    pub fn get_bezier(&self) -> F {
        let y1: F = Signal::get_y1(self.y, One::one());
        let y2: F = y1 + self.bez_depth * y1;
        let relative_val: F = self.val / self.x;
        let ya: F = Signal::get_bezier_pt(Zero::zero(), y2, relative_val);
        let yb: F = Signal::get_bezier_pt(y2, self.y, relative_val);
        Signal::get_bezier_pt(ya, yb, relative_val)
    }

    /// Get oscillator with amplitude and bezier.
    pub fn get_result(&self, val: F) -> F {
        self.amp * val + self.get_bezier()
    }

    /// Get sine wave signal result at val.
    pub fn get_sin(&self) -> F {
        self.get_result((times_two_pi(self.val) * self.freq / self.x).sin())
    }

    /// Get cosine wave signal result at val.
    pub fn get_cos(&self) -> F {
        self.get_result((times_two_pi(self.val) * self.freq / self.x).cos())
    }

    /// Get saw wave signal result at val.
    pub fn get_saw(&self) -> F {
        self.get_result(((self.val * self.freq / self.x) % One::one()) * FromPrimitive::from_int(-2).unwrap() + FromPrimitive::from_int(1).unwrap())
    }
    
    /// Get square wave signal result at val.
    pub fn get_sqr(&self) -> F {
        self.get_result((times_two_pi(self.val) * self.freq / self.x).sin().signum())
    }

    /// Get noise signal result at val.
    pub fn get_noise(&self) -> F {
        self.get_result(get_rand_signal())
    }

    /// Ported implementation of _slang_library_noise1()
    pub fn get_noise_walk(&self) -> F {
        let uno: F = One::one();
        let i0: int = jmath::fast_floor(self.val);
        let i1: int = i0 + 1;
        let x0: F = self.val - FromPrimitive::from_int(i0).unwrap();
        let x1: F = x0 - uno;
        let x12d: F = x1 * x1;
        let x02d: F = x0 * x0;
        let t1: F = uno - x12d;        
        let t0: F = uno - x02d;
        let t0a: F = t0 * t0;
        let g1: f32 = jmath::grad1(jmath::get_perm_val((i0 & 0xff) as uint) as int, x0.to_f32().unwrap());
        let n0: F = t0a * t0a * FromPrimitive::from_f32(g1).unwrap(); 
        let t1a: F = t1 * t1;
        let g2: f32 = jmath::grad1(jmath::get_perm_val((i1 & 0xff) as uint) as int, x1.to_f32().unwrap());
        let n1: F = t1a * t1a * FromPrimitive::from_f32(g2).unwrap();
        let n0pn1: F = n0 + n1;
        let quarter: F = FromPrimitive::from_f32(0.25f32).unwrap();
        quarter * n0pn1
    }
    
}

// Tests
//------------------------------

/*
fn print<F: FloatMath + fmt::Show>(b: Signal<F>) {
    println!("{}", b);
}

pub fn test() {
    println!("Signal struct 'new' test.");
    let mut a = Signal::new(0.0f32);

    print(a.clone());

    println!("Testing sin");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_sin());
    }    
    println!("");
    
    println!("Testing cos");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_cos());
    }    
    println!("");

    println!("Testing get_rand");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_noise());
    }    
    println!("");

    println!("Testing get_sqr");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_sqr());
    }    
    println!("");

    println!("Testing get_saw");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_saw());
    }    
    println!("");

    println!("Testing get_noise_walk");
    for i in range (0.0f32, 20.0f32) {
        a.set_val(jmath::map(i, 0.0f32, 20.0f32, 0.0f32, 1.0f32));
        print!("{}, ", a.get_noise_walk());
    }
    //loop { println!("{}", a.get_noise_walk()); a.set_val(a.val + 0.001f32); }
    println!("");

    println!("Testing set_gradient.");
    a.set_gradient(3.0f32, 1.25f32);
    println!("New gradient = x: {}, y: {}, grad: {}", a.x, a.y, a.grad);
    println!("Testing setters.");
    a.set_freq(20.0f32);
    a.set_amp(0.85f32);
    a.set_bez_depth(0.5f32);
    print(a.clone()); 
}
*/
//------------------------------
