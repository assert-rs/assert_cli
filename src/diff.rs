use std::fmt;

use difference::Difference;
use ansi_term::Colour::{Green, Red};

pub fn render(changeset: &[Difference]) -> Result<String, fmt::Error> {
    use std::fmt::Write;

    let mut t = String::new();

    for change in changeset {
        match *change {
            Difference::Same(ref x) => {
                for line in x.lines() {
                    try!(writeln!(t, " {}", line));
                }
            }
            Difference::Add(ref x) => {
                try!(write!(t, "{}", Green.paint("+")));
                try!(writeln!(t, "{}", Green.paint(x)));
            }
            Difference::Rem(ref x) => {
                try!(write!(t, "{}", Red.paint("-")));
                try!(writeln!(t, "{}", Red.paint(x)));
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
