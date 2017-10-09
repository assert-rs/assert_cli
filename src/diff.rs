extern crate colored;
use self::colored::Colorize;
use difference::{Changeset, Difference};
use std::fmt::{Error as fmtError, Write};

pub fn render(&Changeset { ref diffs, .. }: &Changeset) -> Result<String, fmtError> {
    let mut t = String::new();

    for (i, diff) in diffs.iter().enumerate() {
        match *diff {
            Difference::Same(ref x) => {
                writeln!(t, " {}", x)?;
            }
            Difference::Rem(ref x) => {
                writeln!(t, "{}", format!("-{}", x).red())?;
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        write!(t, "{}", "+".green())?;
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) if !z.is_empty() => {
                                    write!(t, "{}", z.green())?;
                                    write!(t, " ")?;
                                }
                                Difference::Add(ref z) if !z.is_empty() => {
                                    write!(t, "{}", z.green().reverse())?;
                                    write!(t, " ")?;
                                }
                                _ => (),
                            }
                        }
                        writeln!(t, "")?;
                    }
                    _ => {
                        writeln!(t, "{}", format!("+{}", x).green().dimmed())?;
                    }
                };
            }
        }
    }

    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_diff() {
        let diff = Changeset::new("lol", "yay", "\n");
        println!("{}", render(&diff).unwrap());
        assert_eq!(
            render(&diff).unwrap(),
            " \n\u{1b}[31m-lol\u{1b}[0m\n\u{1b}[32m+\u{1b}[0m\u{1b}[7;32myay\u{1b}[0m \n"
        )
    }

    #[test]
    fn multiline_diff() {
        let diff = Changeset::new(
            "Lorem ipsum dolor sit amet, consectetur adipisicing elit,
sed do eiusmod tempor incididunt ut labore et dolore magna
aliqua. Ut enim ad minim veniam, quis nostrud exercitation
ullamco laboris nisi ut aliquip ex ea commodo consequat.",
            "Lorem ipsum dolor sit amet, consectetur adipisicing elit,
sed do eiusmod tempor **incididunt** ut labore et dolore magna
aliqua. Ut enim ad minim veniam, quis nostrud exercitation
ullamco laboris nisi ut aliquip ex ea commodo consequat.",
            "\n",
        );
        println!("{}", render(&diff).unwrap());
        assert_eq!(
            render(&diff).unwrap(),
            " Lorem ipsum dolor sit amet, consectetur adipisicing elit,\n\
             \u{1b}[31m-sed do eiusmod tempor incididunt ut labore et dolore \
             magna\u{1b}[0m\n\u{1b}[32m+\u{1b}[0m\u{1b}[32msed do eiusmod tempor\
             \u{1b}[0m \u{1b}[7;32m**incididunt**\u{1b}[0m \u{1b}[32mut labore \
             et dolore magna\u{1b}[0m \n aliqua. Ut enim ad minim veniam, quis \
             nostrud exercitation\nullamco laboris nisi ut aliquip ex ea \
             commodo consequat.\n"
        );
    }
}
