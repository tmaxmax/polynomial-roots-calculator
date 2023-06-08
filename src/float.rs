pub trait Float {
    fn near_zero(self) -> bool;
    fn negate(self) -> Self;
    fn gcd(self, other: Self) -> Self;
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

    fn gcd(self, other: Self) -> Self {
        match (self == 0., other == 0.) {
            (true, true) => 0.,
            (false, true) => self,
            (true, false) => other,
            _ => {
                let mut r0 = self;
                let mut r1 = other;

                if r0 < r1 {
                    std::mem::swap(&mut r0, &mut r1);
                }

                while r1 != 0. {
                    let rem = r0 % r1;
                    r0 = r1;
                    r1 = rem;
                }

                r0
            }
        }
    }
}
