pub trait Float {
    fn near_zero(self) -> bool;
    fn negate(self) -> Self;
    fn ilog2f(self) -> i32;
}

pub const TOLERANCE: f64 = 1e-15;

impl Float for f64 {
    fn near_zero(self) -> bool {
        -TOLERANCE < self && self < TOLERANCE
    }

    fn negate(self) -> f64 {
        if self == 0. {
            0.
        } else {
            -self
        }
    }

    fn ilog2f(self) -> i32 {
        debug_assert!(self.is_normal());

        let bits = self.to_bits();
        let exp = (bits >> 52) & ((1 << 11) - 1);

        exp as i32 - 1023
    }
}
