#![feature(unboxed_closures, fn_traits, iter_intersperse)]

mod float;
mod polynomial;
mod roots;

use anyhow::Result;
use roots::{find_roots, Roots};
use std::{
    env,
    io::{self, prelude::*, IsTerminal},
};

fn parse_roots<T: AsRef<str>>(iter: impl DoubleEndedIterator<Item = T>) -> Result<Vec<f64>> {
    iter.map(|v| v.as_ref().parse().map_err(anyhow::Error::new))
        .rev()
        .collect()
}

fn parse_stdin(stdin: &mut io::StdinLock) -> Result<Vec<f64>> {
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    parse_roots(buf.split_whitespace())
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

        let coefs = match parse_roots(input.split_whitespace()) {
            Ok(res) => res,
            Err(_) => {
                writeln!(stdout, "\nInvalid input, please try again.")?;
                continue;
            }
        };

        writeln!(
            stdout,
            "{}\nInput coefficients or \"exit\" to close the program.",
            format_output_interactive(&find_roots(&coefs.into()))
        )?;
    }
}

fn format_output_interactive(roots: &Roots) -> String {
    match roots {
        Roots::All => "Real roots: all real numbers".into(),
        Roots::None => "Real roots: none".into(),
        Roots::Some(roots) => roots
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

fn format_output_noninteractive(roots: &Roots) -> String {
    match roots {
        Roots::None => "none".into(),
        Roots::All => "all".into(),
        Roots::Some(roots) => roots
            .iter()
            .map(|r| format!("{}:{}", r.value, r.multiplicity))
            .intersperse(" ".into())
            .collect(),
    }
}

fn main() -> Result<()> {
    let args = env::args();

    let roots;
    if args.len() > 1 {
        roots = parse_roots(args.skip(1))?;
    } else if !io::stdin().is_terminal() {
        roots = parse_stdin(&mut io::stdin().lock())?;
    } else {
        return interactive_prompt(&mut io::stdin().lock(), &mut io::stdout().lock());
    }

    Ok(println!(
        "{}",
        format_output_noninteractive(&find_roots(&roots.into()))
    ))
}
