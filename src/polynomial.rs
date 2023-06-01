use std::{
    fmt::{self, Write},
    ops,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Polynomial(Vec<f64>);

impl Polynomial {
    pub const ZERO: Polynomial = Polynomial(vec![]);

    pub fn iter(&self) -> impl ExactSizeIterator + DoubleEndedIterator<Item = (i32, f64)> + '_ {
        self.0.iter().enumerate().map(|(i, &v)| (i as i32, v))
    }

    pub fn grade(&self) -> i32 {
        (self.0.len() as i32) - 1
    }

    pub fn derivative(&self) -> Polynomial {
        self.iter()
            .skip(1)
            .map(|(i, v)| (i as f64) * v)
            .collect::<Vec<_>>()
            .into()
    }

    pub fn div_rem(&self, rhs: &Polynomial) -> (Polynomial, Polynomial) {
        match rhs.grade() {
            -1 => panic!("Division by 0"),
            0 => (const_div(self, rhs[0]), Self::ZERO),
            1 => horner_div(self, rhs),
            _ => todo!("Implement long division"),
        }
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
    T: AsRef<[f64]>,
{
    fn from(value: T) -> Self {
        let v = value.as_ref();

        if v.len() > i32::MAX as usize {
            panic!("Too many coefficients");
        }

        if v.len() == 1 && v[0] == 0. {
            return Self::ZERO;
        }

        Polynomial(v.into())
    }
}

impl ops::Index<i32> for Polynomial {
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

fn const_div(p: &Polynomial, a: f64) -> Polynomial {
    p.iter().map(|(_, v)| v / a).collect::<Vec<_>>().into()
}

fn horner_div(p: &Polynomial, rhs: &Polynomial) -> (Polynomial, Polynomial) {
    let len = p.0.len();
    let mut res = vec![0.; len];
    res[len - 1] = p[(len - 1) as i32];

    let a = -rhs[0] / rhs[1];

    (0..len - 1)
        .rev()
        .for_each(|k| res[k] = a * res[k + 1] + p[k as i32]);

    res.rotate_left(1);
    let rem = res.pop().unwrap();

    if rhs[1] != 1. {
        res.iter_mut().for_each(|v| *v /= rhs[1]);
    }

    (res.into(), [rem].into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_horner() {
        use super::*;

        let (res, rem) = horner_div(&[2., 1., -2., 8.].into(), &[-1., 2.].into());
        assert_eq!(res, [1., 1., 4.].into());
        assert_eq!(rem[0], 3.);
    }
}
