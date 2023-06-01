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

    pub fn coef(&self, i: i32) -> Option<f64> {
        self.coef_ref(i).copied()
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
            0 => horner_div(self, -rhs[0]),
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
            _ => self.0[0] + self.iter().skip(1).rev().fold(0.0, |a, (_, c)| v * (a + c)),
        }
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Vec<f64>> for Polynomial {
    fn from(value: Vec<f64>) -> Self {
        if value.len() > i32::MAX as usize {
            panic!("Too many coefficients");
        }

        if value.len() == 1 && value[0] == 0. {
            return Self::ZERO;
        }

        Polynomial(value)
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

fn horner_div(p: &Polynomial, a: f64) -> (Polynomial, Polynomial) {
    let len = p.0.len();
    let mut res = vec![0.; len];
    res[len - 1] = p[(len - 1) as i32];

    (0..len - 1)
        .rev()
        .for_each(|k| res[k] = a * res[k + 1] + p[k as i32]);

    res.rotate_left(1);
    let rem = res.pop().unwrap();

    (Polynomial::from(res), Polynomial::from(vec![rem]))
}
