
/// Point for use in the envelope struct.
#[deriving(Clone, Show)]
pub struct Point {
    /// `time` represents the x domain.
    pub time: f32,
    /// `value` represents the y domain.
    pub value: f32,
    /// `curve` represents the bezier curve depth.
    pub curve: f32
}

impl Point {
    /// Constructor method for Point.
    pub fn new(time: f32, value: f32, curve: f32) -> Point {
        Point { time: time, value: value, curve: curve }
    }
}

/// Envelope struct, primarily used for
/// frequency and amplitude interpolation.
#[deriving(Clone, Show)]
pub struct Envelope {
    /// Envelope represented by a vector
    /// of points (sorted by `time`).
    pub points: Vec<Point>
}

impl Envelope {

    /// Default, empty constructor.
    fn new() -> Envelope {
        Envelope {
            points: vec![]
        }
    }

    /// Add a new point to the Envelope.
    fn add_point(&mut self, point: Point) {
        self.points.push(point);
        self.points.sort_by(|a, b| if a.time < b.time { Less }
                                   else if a.time > b.time { Greater }
                                   else { Equal });
    }

    /// Return `value` for the given `time`.
    fn get_value(&self, time: f32) -> f32 {
        // If there is less than two points interpolation
        // is not meaningful, thus we should just return 0.
        if self.points.len() <= 1 { return 0f32 }
        // Iterate through points.
        for i in range(0, self.points.len()) {
            // Find the start point to interpolate.
            if time >= self.points.get(i).time {
                // Interpolate both points and add the value
                // of the first to find our result.
                return self.interpolate(time,
                                        *self.points.get(i-1),
                                        *self.points.get(i))
                    + self.points.get(i-1).value;
            }
        }
        0f32
    }

    /// Interpolate between points.
    fn interpolate(&self, time: f32, start: Point, end: Point) -> f32 {
        // Find time passed from start of interpolation.
        let time_pos = time - start.time;
        // Find duration of interpolation.
        let duration = end.time - start.time;
        // Set gradient for interpolation.
        let gradient_value = end.value - start.value;
        if gradient_value == 0f32 { return 0f32 }
        let gradient = duration / gradient_value;
        let half_gradient_value = gradient_value * 0.5f32;
        // Consider bezier curve.
        let y2 = half_gradient_value + start.curve * half_gradient_value;
        let perc_time = time_pos / duration;
        // Re-adjust linear trajectory.
        let ya = Envelope::get_bezier_pt(0f32, y2, perc_time);
        let yb = Envelope::get_bezier_pt(y2, gradient_value, perc_time);
        Envelope::get_bezier_pt(ya, yb, perc_time)
    }

    /// Get bezier point for bezier curve.
    fn get_bezier_pt(n1: f32, n2: f32, perc: f32) -> f32 {
        (n2 - n1) * perc + n1
    }

}

