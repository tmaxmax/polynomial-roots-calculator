use crate::polynomial::Polynomial;
use std::cmp::Ordering;

pub struct Root {
    pub value: f64,
    pub multiplicity: i32,
}

pub enum Roots {
    All,
    Some(Vec<Root>),
    None,
}

pub fn find_roots(p: &Polynomial) -> Roots {
    match p.grade() {
        -1 => Roots::All,
        0 => Roots::None,
        1 => get_roots_order_one(p),
        2 => get_roots_order_two(p),
        _ => get_roots_general(p),
    }
}

fn negate(f: f64) -> f64 {
    if f == 0. {
        0.
    } else {
        -f
    }
}

fn get_roots_order_one(p: &Polynomial) -> Roots {
    Roots::Some(vec![Root {
        value: negate(p[0]) / p[1],
        multiplicity: 1,
    }])
}

fn get_roots_order_two(p: &Polynomial) -> Roots {
    let two_a = 2. * p[2];
    let delta = p[1] * p[1] - 2. * two_a * p[0];

    match delta.partial_cmp(&0.) {
        Some(o) => match o {
            Ordering::Less => Roots::None,
            Ordering::Equal => Roots::Some(vec![Root {
                value: -p[1] / two_a,
                multiplicity: 2,
            }]),
            Ordering::Greater => Roots::Some(vec![
                Root {
                    value: (-p[1] - delta.sqrt()) / two_a,
                    multiplicity: 1,
                },
                Root {
                    value: (-p[1] + delta.sqrt()) / two_a,
                    multiplicity: 1,
                },
            ]),
        },
        None => Roots::None,
    }
}

fn get_roots_general(p: &Polynomial) -> Roots {
    get_roots_biquadratic(p)
        .or_else(|| get_roots_binomial(p))
        .or_else(|| get_roots_palindrome(p))
        .unwrap_or_else(|| approximate_roots(p))
}

fn get_roots_binomial(p: &Polynomial) -> Option<Roots> {
    use std::f64::consts::PI;

    let grade = p.grade();
    let first = p[0];
    let last = p[grade];

    if (1..grade).map(|i| p[i]).any(|v| v != 0.) {
        return None;
    }

    let abs = (negate(first) / last).abs().powf(1. / (grade as f64));
    let init_phi = (-first.signum()).acos();

    let root_values = (0..grade)
        .flat_map(|k| {
            let phi = (init_phi + PI * (2 * k) as f64) / grade as f64;
            let cos = phi.cos();
            let sin = phi.sin();

            if sin.abs() > 1e-15 {
                return None;
            }

            Some(abs * cos)
        })
        .map(|value| Root {
            value,
            multiplicity: 1,
        })
        .collect::<Vec<_>>();

    if root_values.is_empty() {
        return Some(Roots::None);
    }

    Some(Roots::Some(root_values))
}

fn get_roots_biquadratic(p: &Polynomial) -> Option<Roots> {
    if p.grade() != 4 || p[1] != 0. || p[3] != 0. {
        return None;
    }

    let bp = Polynomial::from(vec![p[0], p[2], p[4]]);

    return Some(match get_roots_order_two(&bp) {
        Roots::Some(roots) => Roots::Some(get_all_roots(roots)),
        _ => Roots::None,
    });

    fn get_all_roots(quadratic_roots: Vec<Root>) -> Vec<Root> {
        quadratic_roots
            .into_iter()
            .flat_map(|r| {
                if r.value >= 0. {
                    let sqrt = r.value.sqrt();

                    return Some(
                        [-sqrt, sqrt]
                            .into_iter()
                            .skip(if r.value > 0. { 0 } else { 1 })
                            .map(move |value| Root {
                                value,
                                multiplicity: r.multiplicity,
                            }),
                    );
                }

                None
            })
            .flatten()
            .collect()
    }
}

fn get_roots_palindrome(_p: &Polynomial) -> Option<Roots> {
    todo!();
}

fn approximate_roots(_p: &Polynomial) -> Roots {
    todo!();
}
