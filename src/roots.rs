use crate::float::Float;
use crate::polynomial::Polynomial;
use std::cmp::Ordering;

pub struct Root {
    pub value: f64,
    pub multiplicity: i32,
}

pub fn find_roots(p: &Polynomial) -> Option<Vec<Root>> {
    match p.grade() {
        -1 => None,
        0 => Some(vec![]),
        1 => Some(get_roots_order_one(p)),
        2 => Some(get_roots_order_two(p)),
        _ => Some(get_roots_general(p)),
    }
}

fn get_roots_order_one(p: &Polynomial) -> Vec<Root> {
    vec![Root {
        value: p[0].negate() / p[1],
        multiplicity: 1,
    }]
}

fn get_roots_order_two(p: &Polynomial) -> Vec<Root> {
    let two_a = 2. * p[2];
    let delta = p[1] * p[1] - 2. * two_a * p[0];

    delta.partial_cmp(&0.).map_or(vec![], |o| match o {
        Ordering::Less => vec![],
        Ordering::Equal => vec![Root {
            value: -p[1] / two_a,
            multiplicity: 2,
        }],
        Ordering::Greater => vec![
            Root {
                value: (-p[1] - delta.sqrt()) / two_a,
                multiplicity: 1,
            },
            Root {
                value: (-p[1] + delta.sqrt()) / two_a,
                multiplicity: 1,
            },
        ],
    })
}

fn get_roots_general(p: &Polynomial) -> Vec<Root> {
    get_roots_biquadratic(p)
        .or_else(|| get_roots_binomial(p))
        .or_else(|| get_roots_palindrome(p))
        .unwrap_or_else(|| approximate_roots(p))
}

fn get_roots_binomial(p: &Polynomial) -> Option<Vec<Root>> {
    use std::f64::consts::PI;

    let grade = p.grade();
    if (1..grade).any(|i| p[i] != 0.) {
        return None;
    }

    let first = p[0];
    let last = p[grade];
    let abs = (-first / last).abs().powf(1. / (grade as f64));
    let init_phi = (-first.signum()).acos();

    let root_values = (0..grade)
        .flat_map(|k| {
            let phi = (init_phi + PI * (2 * k) as f64) / grade as f64;

            phi.sin().abs().near_zero().then(|| abs * phi.cos())
        })
        .map(|value| Root {
            value,
            multiplicity: 1,
        })
        .collect::<Vec<_>>();

    Some(root_values)
}

fn get_roots_biquadratic(p: &Polynomial) -> Option<Vec<Root>> {
    if (p.grade(), p[1], p[3]) != (4, 0., 0.) {
        return None;
    }

    let roots = get_roots_order_two(&[p[0], p[2], p[4]].into())
        .into_iter()
        .filter(|r| r.value < 0.)
        .flat_map(|r| {
            let sqrt = r.value.sqrt();

            [-sqrt, sqrt]
                .into_iter()
                .skip((r.value == 0.) as usize)
                .map(move |value| Root {
                    value,
                    multiplicity: r.multiplicity,
                })
        })
        .collect();

    Some(roots)
}

fn get_roots_palindrome(p: &Polynomial) -> Option<Vec<Root>> {
    return match p.grade() {
        g if g % 2 == 1 && is_palindrome(p) => {
            let mut roots = find_roots(&p.div_rem(&[1., 1.].into()).0)?;
            roots.push(Root {
                value: -1.,
                multiplicity: 1,
            });

            Some(roots)
        }
        4 => get_roots_quartic_quasi_palindrome(p),
        _ => None,
    };

    fn is_palindrome(p: &Polynomial) -> bool {
        p.iter().all(|(i, v)| v == p[p.grade() - i])
    }

    fn get_roots_quartic_quasi_palindrome(p: &Polynomial) -> Option<Vec<Root>> {
        let m = (p[0] / p[4]).sqrt();
        let m2 = p[1] / p[3];

        if m != m2 {
            return None;
        }

        let roots = get_roots_order_two(&[p[2] - 2. * p[4] * m, p[3], p[4]].into())
            .into_iter()
            .flat_map(|r| {
                get_roots_order_two(&[m, -r.value, 1.].into())
                    .into_iter()
                    .map(move |mut qr| {
                        qr.multiplicity *= r.multiplicity;
                        qr
                    })
            })
            .collect();

        Some(roots)
    }
}

fn approximate_roots(_p: &Polynomial) -> Vec<Root> {
    todo!("roots approximation algorithm");
}
