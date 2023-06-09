use std::{
    fmt::{self, Write},
    ops::Index,
};

use num_rational::Rational32;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, PartialEq)]
pub struct Polynomial(Vec<f64>);

impl Polynomial {
    pub const ZERO: Self = Self(vec![]);

    pub fn iter(&self) -> impl ExactSizeIterator + DoubleEndedIterator<Item = (i32, f64)> + '_ {
        self.0.iter().enumerate().map(|(i, &v)| (i as i32, v))
    }

    pub fn grade(&self) -> i32 {
        (self.0.len() as i32) - 1
    }

    pub fn derivative(&self) -> Self {
        Self(self.iter().skip(1).map(|(i, v)| (i as f64) * v).collect())
    }

    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        let (res, rem) = div(self.to_ratios(), &rhs.to_ratios());
        (Polynomial::from_ratios(res), Polynomial::from_ratios(rem))
    }

    pub fn lead(&self) -> f64 {
        self[self.grade()]
    }

    pub fn primitive(&self) -> (Polynomial, f64) {
        let mut r = self.to_ratios();
        let d = primitive(&mut r);

        (
            Polynomial::from_ratios(r),
            *d.numer() as f64 / *d.denom() as f64,
        )
    }

    pub fn gcd(&self, rhs: &Self) -> Self {
        match (self.grade(), rhs.grade()) {
            (0, 0) => Self::ZERO,
            (_, 0) => self.clone(),
            (0, _) => rhs.clone(),
            _ => Self::from_ratios(gcd(self.to_ratios(), rhs.to_ratios())),
        }
    }

    pub fn gsfd(&self) -> Self {
        match self.grade() {
            -1 | 0 => self.clone(),
            _ => {
                let s = self.to_ratios();
                let g = gcd(s.clone(), self.derivative().to_ratios());

                let mut res = long_div(s, &g).0;
                primitive(&mut res);

                Polynomial::from_ratios(res)
            }
        }
    }

    pub fn is_palindrome(&self) -> bool {
        self.iter().all(|(i, v)| v == self[self.grade() - i])
    }

    pub fn root_bound(&self) -> Option<f64> {
        let n = self.grade();
        let lead_abs = self.lead().abs();
        let k = lead_abs.log2().ceil() as i32;

        (1..=n)
            .map(|i| {
                let h = self[n - i].abs().log2().ceil() as i32;
                let l = (h - k - 1) / i;

                if (k - h) % i == 2 % i {
                    l + 3
                } else {
                    l + 2
                }
            })
            .max()
            .map(|v| 2f64.powi(v))
    }

    fn coef_ref(&self, i: i32) -> Option<&f64> {
        self.0.get(i as usize).or_else(|| {
            if i == 0 && self.grade() == -1 {
                return Some(&0.);
            }
            None
        })
    }

    fn evaluate(&self, v: f64) -> f64 {
        match self.grade() {
            -1 => 0.,
            _ => self.0[0] + self.iter().skip(1).rev().fold(0., |a, (_, c)| v * (a + c)),
        }
    }

    fn to_ratios(&self) -> Vec<Rational32> {
        self.0
            .iter()
            .map(|&v| Rational32::from_f64(v))
            .collect::<Option<_>>()
            .expect("values too big")
    }

    fn from_ratios(r: Vec<Rational32>) -> Self {
        Self(
            r.into_iter()
                .map(|v| *v.numer() as f64 / *v.denom() as f64)
                .collect(),
        )
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<T> From<T> for Polynomial
where
    T: Into<Vec<f64>>,
{
    fn from(value: T) -> Self {
        let v: Vec<_> = value.into();

        if v.len() > i32::MAX as usize {
            panic!("Too many coefficients");
        }

        if v.iter().any(|v| !v.is_finite()) {
            panic!("Coefficients are not finite floats");
        }

        if v.len() == 1 && v[0] == 0. {
            return Self::ZERO;
        }

        Self(v)
    }
}

impl Index<i32> for Polynomial {
    type Output = f64;

    fn index(&self, i: i32) -> &Self::Output {
        self.coef_ref(i).unwrap()
    }
}

impl FnOnce<(f64,)> for Polynomial {
    type Output = f64;

    extern "rust-call" fn call_once(self, args: (f64,)) -> Self::Output {
        self.evaluate(args.0)
    }
}

impl FnMut<(f64,)> for Polynomial {
    extern "rust-call" fn call_mut(&mut self, args: (f64,)) -> Self::Output {
        self.evaluate(args.0)
    }
}

impl Fn<(f64,)> for Polynomial {
    extern "rust-call" fn call(&self, args: (f64,)) -> Self::Output {
        self.evaluate(args.0)
    }
}

impl fmt::Display for Polynomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.grade() == -1 {
            return f.write_char('0');
        }

        let s = self
            .iter()
            .flat_map(|(i, v)| format_coefficient(v, i, "x", i == self.grade()))
            .rev()
            .collect::<String>();

        f.write_str(&s)
    }
}

fn format_coefficient(v: f64, pow: i32, var: &str, first: bool) -> Option<String> {
    if v == 0.0 {
        return None;
    }

    let mut ret = String::new();

    if !first && v >= 0. {
        ret += "+";
    }

    if v != 1.0 || pow == 0 {
        ret += v.to_string().as_ref();
    }

    if pow > 0 {
        ret += var;
    }

    if pow > 1 {
        ret += "^";
        ret += pow.to_string().as_ref();
    }

    Some(ret)
}

const ZERO: Rational32 = Rational32::new_raw(0, 1);
const ONE: Rational32 = Rational32::new_raw(1, 1);

fn horner_div(mut lhs: Vec<Rational32>, rhs: &[Rational32]) -> (Vec<Rational32>, Rational32) {
    let a = -rhs[0] / rhs[1];

    (0..lhs.len() - 1).rev().for_each(|k| {
        let prev = lhs[k + 1];
        lhs[k] += a * prev;
    });

    lhs.rotate_left(1);
    let rem = lhs.pop().unwrap();

    if rhs[1] != ONE {
        lhs.iter_mut().for_each(|v| *v /= rhs[1]);
    }

    (lhs, rem)
}

fn long_div(mut lhs: Vec<Rational32>, rhs: &[Rational32]) -> (Vec<Rational32>, Vec<Rational32>) {
    let init_l_grade = lhs.len() - 1;
    let init_r_grade = rhs.len() - 1;
    if init_l_grade < init_r_grade {
        return (vec![], lhs);
    }

    let res_g = init_l_grade - init_r_grade;
    let mut res = vec![ZERO; res_g + 1];

    while lhs.len() >= rhs.len() {
        let l_g = lhs.len() - 1;
        let r_g = rhs.len() - 1;
        let c = lhs[l_g] / rhs[r_g];

        (0..=r_g).for_each(|k| lhs[l_g - k] -= c * rhs[r_g - k]);

        while let Some(v) = lhs.last() {
            if *v != ZERO {
                break;
            }

            lhs.pop();
        }

        res[res_g - (init_l_grade - l_g)] = c;
    }

    (res, lhs)
}

fn div(mut lhs: Vec<Rational32>, rhs: &[Rational32]) -> (Vec<Rational32>, Vec<Rational32>) {
    match rhs.len() {
        0 => panic!("Division by 0"),
        1 => {
            lhs.iter_mut().for_each(|v| *v /= rhs[0]);
            (lhs, vec![])
        }
        2 => {
            let (res, rem) = horner_div(lhs, rhs);
            (res, if rem == ZERO { vec![] } else { vec![rem] })
        }
        _ => long_div(lhs, rhs),
    }
}

fn gcd(mut r0: Vec<Rational32>, mut r1: Vec<Rational32>) -> Vec<Rational32> {
    if r0.len() < r1.len() {
        std::mem::swap(&mut r0, &mut r1);
    }

    while !r1.is_empty() {
        let (_, rem) = div(r0, &r1);
        r0 = r1;
        r1 = rem;
    }

    primitive(&mut r0);

    r0
}

fn primitive(v: &mut [Rational32]) -> Rational32 {
    let mut d = v.iter().fold(ZERO, |acc, &v| gcd(acc, v));
    if opposite_signs(v.last().unwrap(), &d) {
        d = -d;
    }

    v.iter_mut().for_each(|v| *v /= d);

    return d;

    fn gcd(mut a: Rational32, mut b: Rational32) -> Rational32 {
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }

        while b != ZERO {
            let rem = a % b;
            a = b;
            b = rem;
        }

        a
    }

    fn opposite_signs(a: &Rational32, b: &Rational32) -> bool {
        (*a.numer() ^ *b.numer()) < 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horner() {
        let (res, rem) = Polynomial::from([2., 1., -2., 8.]).div_rem(&[-1., 2.].into());
        assert_eq!(res, [1., 1., 4.].into());
        assert_eq!(rem, [3.].into());
    }

    #[test]
    fn test_long_div() {
        let (res, rem) = Polynomial::from([2., 1., 0., 2., 1.]).div_rem(&[1., 1., 1.].into());
        assert_eq!(res, [-2., 1., 1.].into());
        assert_eq!(rem, [4., 2.].into());

        let (res, rem) = Polynomial::from([1., 0., 1., 0., 1., 1.]).div_rem(&[1., 0., 1.].into());
        assert_eq!(res, [0., -1., 1., 1.].into());
        assert_eq!(rem, [1., 1.].into());

        let (res, rem) = Polynomial::from([1., 2., 3., 2., 1.]).div_rem(&[1., 1., 1.].into());
        assert_eq!(res, [1., 1., 1.].into());
        assert_eq!(rem, Polynomial::ZERO);
    }

    #[test]
    fn test_gcd() {
        let res = Polynomial::from([0., -2., 1.]).gcd(&[-4., -2., 0., 1.].into());
        assert_eq!(res, [-2., 1.].into());

        let res = Polynomial::from([4., -3., 1., -3., 1.]).gcd(&[-1., 0., 0., 1.].into());
        assert_eq!(res, [-1., 1.].into());

        let res = Polynomial::from([1., 1., 1.]).gcd(&[1., 2., 1.].into());
        assert_eq!(res, [1.].into());
    }

    #[test]
    fn test_gsfd() {
        let a: Polynomial = [1., 2., 1.].into();
        assert_eq!(a.gsfd(), [1., 1.].into());

        let a: Polynomial = [1., 2., 3., 2., 1.].into();
        assert_eq!(a.gsfd(), [1., 1., 1.].into());

        let a: Polynomial = [1875., -2000., -1025., 640., 425., 80., 5.].into(); // 5(x-1)^2(x+3)(x+5)^3
        assert_eq!(a.gsfd(), [-15., 7., 7., 1.].into()); // (x-1)(x+3)(x+5)
    }

    #[test]
    fn test_primitive() {
        let a: Polynomial = [2., -4., -4.].into();
        assert_eq!(a.primitive(), ([-1., 2., 2.].into(), -2.));
    }

    use rand::Rng;

    #[bench]
    fn bench_to_rational(b: &mut test::Bencher) {
        let p = Polynomial(
            rand::thread_rng()
                .sample_iter(rand::distributions::Uniform::from(-1000..1000))
                .map(|v| v as f64)
                .take(1000)
                .collect(),
        );

        b.iter(|| p.to_ratios());
    }

    #[bench]
    fn bench_from_rational(b: &mut test::Bencher) {
        let mut rng = rand::thread_rng();
        let r: Vec<_> = std::iter::from_fn(|| {
            Some(Rational32::new_raw(
                rng.gen_range(-1000..1000),
                rng.gen_range(1..1000),
            ))
        })
        .take(1000)
        .collect();

        b.iter(|| Polynomial::from_ratios(r.clone()))
    }
}
