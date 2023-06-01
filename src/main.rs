#![feature(unboxed_closures, fn_traits, iter_intersperse)]

mod float;
mod polynomial;
mod roots;

use anyhow::Result;
use atty::Stream;
use polynomial::Polynomial;
use roots::{find_roots, Roots};
use std::{env, io, io::prelude::*};

fn parse_roots<T: AsRef<str>>(iter: impl DoubleEndedIterator<Item = T>) -> Result<Vec<f64>> {
    iter.map(|v| v.as_ref().parse().map_err(anyhow::Error::new))
        .rev()
        .collect()
}

fn interactive_prompt(stdin: &mut io::Stdin, stdout: &mut io::Stdout) -> Result<Vec<f64>> {
    println!("Welcome to the polynomial real roots calculator!");
    println!("Please type in the coefficients, from the highest to the lowest monomial. Press Enter when ready.");
    print!("> ");
    stdout.flush()?;

    let mut buf = String::new();
    stdin.lock().read_line(&mut buf)?;

    parse_roots(buf.split_whitespace())
}

fn parse_stdin(stdin: &mut io::Stdin) -> Result<Vec<f64>> {
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;

    parse_roots(buf.split_whitespace())
}

fn main() -> Result<()> {
    let args = env::args();

    let roots;
    if args.len() > 1 {
        roots = parse_roots(args.skip(1))?
    } else if atty::is(Stream::Stdin) && atty::is(Stream::Stdout) {
        roots = interactive_prompt(&mut io::stdin(), &mut io::stdout())?
    } else {
        roots = parse_stdin(&mut io::stdin())?
    }

    let p: Polynomial = roots.into();
    let d = p.derivative();

    println!("Polynomial: {p}");
    println!("Derivative: {d}");

    match find_roots(&p) {
        Roots::All => println!("Real roots: all real numbers"),
        Roots::None => println!("Real roots: none"),
        Roots::Some(roots) => println!(
            "Real roots: {}.",
            roots
                .into_iter()
                .map(|r| format!(
                    "{}{}",
                    r.value,
                    if r.multiplicity > 1 {
                        format!(" (mul. {})", r.multiplicity)
                    } else {
                        "".into()
                    }
                ))
                .intersperse(", ".into())
                .collect::<String>()
        ),
    }

    Ok(())
}
