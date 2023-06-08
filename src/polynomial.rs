use std::{
    fmt::{self, Write},
    ops::{Add, Index, Neg, Sub},
};

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

    pub fn div_rem(mut self, rhs: &Self) -> (Self, Self) {
        match rhs.grade() {
            -1 => panic!("Division by 0"),
            0 => {
                const_div(&mut self, rhs[0]);
                (self, Self::ZERO)
            }
            1 => {
                let (res, rem) = horner_div(self, rhs);
                (res, if rem == 0. { Self::ZERO } else { [rem].into() })
            }
            _ => long_div(self, rhs),
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

impl Add<&Polynomial> for &Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: &Polynomial) -> Self::Output {
        add(self.clone(), rhs)
    }
}

impl Add<&Polynomial> for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: &Polynomial) -> Self::Output {
        add(self, rhs)
    }
}

impl Add<Polynomial> for &Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: Polynomial) -> Self::Output {
        add(rhs, self)
    }
}

impl Add<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: Polynomial) -> Self::Output {
        add(self, &rhs)
    }
}

impl Sub<&Polynomial> for &Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: &Polynomial) -> Self::Output {
        sub(self.clone(), rhs)
    }
}

impl Sub<&Polynomial> for Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: &Polynomial) -> Self::Output {
        sub(self, rhs)
    }
}

impl Sub<Polynomial> for &Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: Polynomial) -> Self::Output {
        sub_inv(self, rhs)
    }
}

impl Sub<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: Polynomial) -> Self::Output {
        sub(self, &rhs)
    }
}

impl Neg for Polynomial {
    type Output = Polynomial;

    fn neg(mut self) -> Self::Output {
        self.0.iter_mut().for_each(|v| *v = -*v);
        self
    }
}

impl Neg for &Polynomial {
    type Output = Polynomial;

    fn neg(self) -> Self::Output {
        self.0.iter().map(|v| -v).collect::<Vec<_>>().into()
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

fn add(mut lhs: Polynomial, rhs: &Polynomial) -> Polynomial {
    lhs.0
        .iter_mut()
        .zip(rhs.0.iter())
        .for_each(|(v, r)| *v += r);

    if rhs.grade() > lhs.grade() {
        lhs.0.extend(rhs.0.iter().skip(lhs.0.len()));
    }

    lhs
}

fn sub(mut lhs: Polynomial, rhs: &Polynomial) -> Polynomial {
    lhs.0
        .iter_mut()
        .zip(rhs.0.iter())
        .for_each(|(l, r)| *l -= r);

    if rhs.grade() > lhs.grade() {
        lhs.0.extend(rhs.0.iter().skip(lhs.0.len()).map(|v| -v));
    }

    lhs
}

fn sub_inv(lhs: &Polynomial, mut rhs: Polynomial) -> Polynomial {
    rhs.0
        .iter_mut()
        .zip(lhs.0.iter())
        .for_each(|(r, l)| *r = l - *r);

    if lhs.grade() > rhs.grade() {
        rhs.0.extend(lhs.0.iter().skip(rhs.0.len()));
    }

    rhs
}

fn const_div(lhs: &mut Polynomial, a: f64) {
    lhs.0.iter_mut().for_each(|v| *v /= a);
}

fn horner_div(mut lhs: Polynomial, rhs: &Polynomial) -> (Polynomial, f64) {
    let a = -rhs[0] / rhs[1];

    (0..lhs.0.len() - 1)
        .rev()
        .for_each(|k| lhs.0[k] += a * lhs.0[k + 1]);

    lhs.0.rotate_left(1);
    let rem = lhs.0.pop().unwrap();

    if rhs[1] != 1. {
        lhs.0.iter_mut().for_each(|v| *v /= rhs[1]);
    }

    (lhs, rem)
}

fn long_div(mut lhs: Polynomial, rhs: &Polynomial) -> (Polynomial, Polynomial) {
    let init_grade = lhs.grade();
    if init_grade < rhs.grade() {
        return (Polynomial::ZERO, lhs);
    }

    let res_g = (init_grade - rhs.grade()) as usize;
    let mut res = vec![0.; res_g + 1];

    while lhs.grade() >= rhs.grade() {
        let l_g = lhs.grade() as usize;
        let r_g = rhs.grade() as usize;
        let a = lhs.0[l_g];
        let b = rhs.0[r_g];
        let c = a / b;

        (0..=r_g).for_each(|k| lhs.0[l_g - k] -= a * rhs.0[r_g - k] / b);

        while let Some(v) = lhs.0.last() {
            if !v.near_zero() {
                break;
            }

            lhs.0.pop();
        }

        res[res_g - (init_grade as usize - l_g)] = c;
    }

    (res.into(), lhs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horner() {
        let (res, rem) = horner_div([2., 1., -2., 8.].into(), &[-1., 2.].into());
        assert_eq!(res, [1., 1., 4.].into());
        assert_eq!(rem, 3.);
    }

    #[test]
    fn test_long_div() {
        let (res, rem) = long_div([2., 1., 0., 2., 1.].into(), &[1., 1., 1.].into());
        assert_eq!(res, [-2., 1., 1.].into());
        assert_eq!(rem, [4., 2.].into());

        let (res, rem) = long_div([1., 0., 1., 0., 1., 1.].into(), &[1., 0., 1.].into());
        assert_eq!(res, [0., -1., 1., 1.].into());
        assert_eq!(rem, [1., 1.].into());

        let (res, rem) = long_div([1., 2., 3., 2., 1.].into(), &[1., 1., 1.].into());
        assert_eq!(res, [1., 1., 1.].into());
        assert_eq!(rem, Polynomial::ZERO);
    }

    #[test]
    fn test_add_sub() {
        let a: Polynomial = [1., 1.].into();
        let b: Polynomial = [1., 2., 4.].into();

        assert_eq!(Polynomial::from([2., 3., 4.]), &a + &b);
        assert_eq!(Polynomial::from([2., 3., 4.]), &b + &a);
        assert_eq!(Polynomial::from([0., -1., -4.]), &a - &b);
        assert_eq!(Polynomial::from([0., 1., 4.]), &b - a);
    }
}
