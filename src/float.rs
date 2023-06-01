pub trait Float {
    fn near_zero(self) -> bool;
    fn negate(self) -> Self;
}

pub const TOLERANCE: f64 = 1e-15;

impl Float for f64 {
    fn near_zero(self) -> bool {
        self < TOLERANCE
    }

    fn negate(self) -> f64 {
        if self == 0. {
            0.
        } else {
            -self
        }
    }
}
