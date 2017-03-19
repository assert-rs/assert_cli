extern crate colored;
use self::colored::Colorize;

use difference::{Difference, Changeset};
use std::fmt::Write;

use errors::*;

pub fn render(Changeset { diffs, .. }: Changeset) -> Result<String> {
    let mut t = String::new();

    for i in 0..diffs.len() {
        match diffs[i] {
            Difference::Same(ref x) => {
                writeln!(t, " {}", x)?;
            }
            Difference::Add(ref x) => {
                match diffs[i - 1] {
                    Difference::Rem(ref y) => {
                        write!(t, "{}", "+".green())?;
                        let Changeset { diffs, .. } = Changeset::new(y, x, " ");
                        for c in diffs {
                            match c {
                                Difference::Same(ref z) => {
                                    write!(t, "{}", z.green())?;
                                    write!(t, " ")?;
                                }
                                Difference::Add(ref z) => {
                                    write!(t, "{}", z.white().on_green())?;
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
            Difference::Rem(ref x) => {
                writeln!(t, "{}", format!("-{}", x).red())?;
            }
        }
    }

    Ok(t)
}

#[cfg(test)]
mod tests {
    use difference::diff;
    use super::*;

    #[test]
    fn basic_diff() {
        let (_, diff) = diff("lol", "yay", "\n");
        assert_eq!(render(&diff).unwrap(),
                   "\u{1b}[31m-\u{1b}[0m\u{1b}[31mlol\u{1b}[0m\n\u{1b}[32m+\u{1b}[0m\u{1b}[32myay\
                    \u{1b}[0m\n")
    }

    #[test]
    fn multiline_diff() {
        let (_, diff) = diff("Lorem ipsum dolor sit amet, consectetur adipisicing elit,
sed do eiusmod tempor incididunt ut labore et dolore magna
aliqua. Ut enim ad minim veniam, quis nostrud exercitation
ullamco laboris nisi ut aliquip ex ea commodo consequat.",
                             "Lorem ipsum dolor sit amet, consectetur adipisicing elit,
sed do eiusmod tempor **incididunt** ut labore et dolore magna
aliqua. Ut enim ad minim veniam, quis nostrud exercitation
ullamco laboris nisi ut aliquip ex ea commodo consequat.",
                             "\n");
        assert_eq!(render(&diff).unwrap(), " Lorem ipsum dolor sit amet, consectetur adipisicing elit,\n\u{1b}[31m-\u{1b}[0m\u{1b}[31msed do eiusmod tempor incididunt ut labore et dolore magna\u{1b}[0m\n\u{1b}[32m+\u{1b}[0m\u{1b}[32msed do eiusmod tempor **incididunt** ut labore et dolore magna\u{1b}[0m\n aliqua. Ut enim ad minim veniam, quis nostrud exercitation\n ullamco laboris nisi ut aliquip ex ea commodo consequat.\n");
    }
}
