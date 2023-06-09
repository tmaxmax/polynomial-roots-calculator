#![feature(unboxed_closures, fn_traits, iter_intersperse, test)]

extern crate test;

mod float;
mod polynomial;
mod roots;

use anyhow::Result;
use polynomial::Polynomial;
use roots::{find_roots, Root};
use std::{
    env,
    io::{self, prelude::*, IsTerminal},
};

fn parse_coefs(iter: impl DoubleEndedIterator<Item = impl AsRef<str>>) -> Result<Vec<f64>> {
    iter.map(|v| v.as_ref().parse().map_err(anyhow::Error::new))
        .rev()
        .collect()
}

fn parse_stdin(stdin: &mut io::StdinLock) -> Result<Vec<f64>> {
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    parse_coefs(buf.split_whitespace())
}

fn interactive_prompt(stdin: &mut io::StdinLock, stdout: &mut io::StdoutLock) -> Result<()> {
    writeln!(stdout, "Welcome to the polynomial real roots calculator")?;
    writeln!(stdout, "Please type in the coefficients, from the highest to the lowest monomial. Press Enter when ready.")?;

    let mut input = String::new();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        input.clear();
        stdin.read_line(&mut input)?;

        if input.trim() == "exit" {
            writeln!(stdout, "Bye!")?;
            return Ok(());
        }

        let coefs = match parse_coefs(input.split_whitespace()) {
            Ok(res) => res,
            Err(_) => {
                writeln!(stdout, "\nInvalid input, please try again.")?;
                continue;
            }
        };
        let p: Polynomial = coefs.into();

        writeln!(
            stdout,
            "Polynomial: {}\nDerivative: {}\nRoot bound: {}\nRoots: {}\n\nInput coefficients or \"exit\" to close the program.",
            p,
            p.derivative(),
            p.root_bound().map_or("none".into(), |v| format!("±{v} (approx.)")),
            format_output_interactive(find_roots(&p).as_deref())
        )?;
    }
}

fn format_output_interactive(roots: Option<&[Root]>) -> String {
    match roots {
        None => "Real roots: zero polynomial".into(),
        Some([]) => "Real roots: none".into(),
        Some(roots) => roots
            .iter()
            .map(|r| {
                format!(
                    "{}{}",
                    r.value,
                    if r.multiplicity > 1 {
                        format!(" (mul. {})", r.multiplicity)
                    } else {
                        "".into()
                    }
                )
            })
            .intersperse(", ".into())
            .collect(),
    }
}

fn format_output_noninteractive(roots: Option<&[Root]>) -> String {
    match roots {
        None => "zero".into(),
        Some([]) => "none".into(),
        Some(roots) => roots
            .iter()
            .map(|r| format!("{}:{}", r.value, r.multiplicity))
            .intersperse(" ".into())
            .collect(),
    }
}

fn main() -> Result<()> {
    let args = env::args();

    let coefs;
    if args.len() > 1 {
        coefs = parse_coefs(args.skip(1))?;
    } else if !io::stdin().is_terminal() {
        coefs = parse_stdin(&mut io::stdin().lock())?;
    } else {
        return interactive_prompt(&mut io::stdin().lock(), &mut io::stdout().lock());
    }

    Ok(println!(
        "{}",
        format_output_noninteractive(find_roots(&coefs.into()).as_deref())
    ))
}
