use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Write},
    ops::Index,
};

use num_rational::{BigRational, Rational32};

use crate::float::Float;

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
        self.iter()
            .skip(1)
            .map(|(i, v)| (i as f64) * v)
            .collect::<Vec<_>>()
            .into()
    }

    pub fn div_rem(&self, rhs: &Self) -> (Self, Self) {
        match rhs.grade() {
            -1 => panic!("Division by 0"),
            0 => {
                let mut res = self.clone();
                const_div(&mut res, rhs[0]);

                (res, Self::ZERO)
            }
            _ => {
                let (res, rem) = div(self.to_ratios(), &rhs.to_ratios());
                (Polynomial::from_ratios(&res), Polynomial::from_ratios(&rem))
            }
        }
    }

    pub fn lead(&self) -> f64 {
        self[self.grade()]
    }

    pub fn primitive(mut self) -> (Polynomial, f64) {
        let d = self.0.iter().fold(0., |acc, v| acc.gcd(*v));
        self.0.iter_mut().for_each(|v| *v /= d);

        (self, d)
    }

    pub fn gcd(&self, rhs: &Self) -> Self {
        match (self.grade(), rhs.grade()) {
            (0, 0) => Self::ZERO,
            (_, 0) => self.clone(),
            (0, _) => rhs.clone(),
            _ => Self::from_ratios(&gcd(self.to_ratios(), rhs.to_ratios())),
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

                Polynomial::from_ratios(&res)
            }
        }
    }

    pub fn is_palindrome(&self) -> bool {
        self.iter().all(|(i, v)| v == self[self.grade() - i])
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
            .map(|&v| -> Result<_, Box<dyn Error>> {
                let r = BigRational::from_float(v).expect("float must be finite");
                let n: i32 = r.numer().try_into()?;
                let d: i32 = r.denom().try_into()?;

                Ok(Rational32::new(n, d))
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn from_ratios(r: &[Rational32]) -> Self {
        r.iter()
            .map(|v| *v.numer() as f64 / *v.denom() as f64)
            .collect::<Vec<_>>()
            .into()
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

fn const_div(lhs: &mut Polynomial, a: f64) {
    lhs.0.iter_mut().for_each(|v| *v /= a);
}

fn horner_div(mut lhs: Vec<Rational32>, rhs: &[Rational32]) -> (Vec<Rational32>, Rational32) {
    let a = -rhs[0] / rhs[1];

    (0..lhs.len() - 1).rev().for_each(|k| {
        let prev = lhs[k + 1];
        lhs[k] += a * prev;
    });

    lhs.rotate_left(1);
    let rem = lhs.pop().unwrap();

    if rhs[1] != Rational32::new(1, 1) {
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
    let mut res = vec![Rational32::default(); res_g + 1];

    while lhs.len() >= rhs.len() {
        let l_g = lhs.len() - 1;
        let r_g = rhs.len() - 1;
        let c = lhs[l_g] / rhs[r_g];

        (0..=r_g).for_each(|k| lhs[l_g - k] -= c * rhs[r_g - k]);

        while let Some(v) = lhs.last() {
            if *v != Rational32::default() {
                break;
            }

            lhs.pop();
        }

        res[res_g - (init_l_grade - l_g)] = c;
    }

    (res, lhs)
}

fn div(lhs: Vec<Rational32>, rhs: &[Rational32]) -> (Vec<Rational32>, Vec<Rational32>) {
    match lhs.len() {
        0 | 1 => unreachable!(),
        2 => {
            let (res, rem) = horner_div(lhs, rhs);
            (
                res,
                if rem == Rational32::default() {
                    vec![]
                } else {
                    vec![rem]
                },
            )
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

    r0
}

fn primitive(v: &mut [Rational32]) -> Rational32 {
    let mut d = v.iter().fold(Rational32::default(), |acc, &v| gcd(acc, v));
    if sgn(v.last().unwrap()) * sgn(&d) < 0 {
        d = -d;
    }

    v.iter_mut().for_each(|v| *v /= d);

    return d;

    fn gcd(mut a: Rational32, mut b: Rational32) -> Rational32 {
        if a < b {
            std::mem::swap(&mut a, &mut b);
        }

        while b != Rational32::default() {
            let rem = a % b;
            a = b;
            b = rem;
        }

        a
    }

    fn sgn(r: &Rational32) -> i32 {
        match r.cmp(&Rational32::default()) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
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
        assert_eq!(res, [-4., 2.].into());

        let res = Polynomial::from([4., -3., 1., -3., 1.]).gcd(&[-1., 0., 0., 1.].into());
        assert_eq!(res, [-3., 3.].into());
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
}
