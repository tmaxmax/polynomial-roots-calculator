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
    if let Some(b) = get_binomial(p) {
        return get_roots_binomial(b);
    }

    if let Some(q) = get_biquadratic(p) {
        if let Roots::Some(roots) = get_roots_order_two(&q) {
            return Roots::Some(
                roots
                    .into_iter()
                    .flat_map(gen_roots_biquadratic)
                    .flatten()
                    .collect(),
            );
        }
    }

    approximate_roots(p)
}

fn approximate_roots(_p: &Polynomial) -> Roots {
    // TODO: Implement root finding algorithm.
    Roots::None
}

#[derive(Debug)]
struct Binomial {
    grade: i32,
    coefs: (f64, f64),
}

fn get_binomial(p: &Polynomial) -> Option<Binomial> {
    let grade = p.grade();
    let first = p[0];
    let last = p[grade];

    if (1..p.grade()).map(|i| p[i]).any(|v| v != 0.) {
        return None;
    }

    Some(Binomial {
        grade,
        coefs: (first, last),
    })
}

fn get_roots_binomial(b: Binomial) -> Roots {
    use std::f64::consts::PI;

    let abs = (negate(b.coefs.0) / b.coefs.1)
        .abs()
        .powf(1. / (b.grade as f64));
    let init_phi = (-b.coefs.0.signum()).acos();

    println!("b={b:?} phi={init_phi}");

    let root_values = (0..b.grade)
        .flat_map(|k| {
            let phi = (init_phi + PI * (2 * k) as f64) / b.grade as f64;
            let cos = phi.cos();
            let sin = phi.sin();

            println!("{k}: phi={phi} cos={cos} sin={sin}");

            if sin.abs() > 1e-15 {
                return None;
            }

            Some(abs * cos)
        })
        .collect::<Vec<_>>();

    if root_values.is_empty() {
        return Roots::None;
    }

    let mut roots: Vec<Root> = vec![];

    for value in root_values {
        if let Some(root) = roots.iter_mut().find(|r| r.value == value) {
            root.multiplicity += 1;
        } else {
            roots.push(Root {
                value,
                multiplicity: 1,
            })
        }
    }

    Roots::Some(roots)
}

fn get_biquadratic(p: &Polynomial) -> Option<Polynomial> {
    if p.grade() != 4 || p[1] != 0. || p[3] != 0. {
        return None;
    }

    Some(Polynomial::from(vec![p[0], p[2], p[4]]))
}

fn gen_roots_biquadratic(r: Root) -> Option<impl Iterator<Item = Root>> {
    if r.value >= 0. {
        let sqrt = r.value.sqrt();

        return Some([-sqrt, sqrt].into_iter().map(move |value| Root {
            value,
            multiplicity: r.multiplicity,
        }));
    }

    None
}
