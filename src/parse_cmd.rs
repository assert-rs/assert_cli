pub trait ToCmd<'a> {
    fn to_cmd(&'a self) -> Vec<String>;
}

impl<'a> ToCmd<'a> for str {
    fn to_cmd(&'a self) -> Vec<String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_quote = Vec::new();

        for c in self.chars() {
            if in_quote.is_empty() && c.is_whitespace() {
                args.push(current_arg);
                current_arg = String::new();
                continue;
            }

            current_arg.push(c);

            if c == '"' || c == '\'' {
                if in_quote.last() == Some(&c) {
                    in_quote.pop();
                } else {
                    in_quote.push(c);
                }
            }
        }

        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        args
    }
}

impl<'a, 'b, T> ToCmd<'a> for T where
    &'a T: AsRef<[&'b str]>,
    T: 'a,
{
    fn to_cmd(&'a self) -> Vec<String> {
        self.as_ref().into_iter().map(|x| x.to_string()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::ToCmd;

    #[test]
    fn slices() {
        assert_eq!(
            ToCmd::to_cmd(&["echo", "42"]),
            vec!["echo", "42"]
        );
    }

    #[test]
    fn simple() {
        assert_eq!(
            "echo 42".to_cmd(),
            vec!["echo", "42"]
        );
        assert_eq!(
            r#"echo "42""#.to_cmd(),
            vec!["echo", "\"42\""]
        );
        assert_eq!(
            r#"echo '42'"#.to_cmd(),
            vec!["echo", "\'42\'"]
        );
        assert_eq!(
            r#"echo '42 is the answer'"#.to_cmd(),
            vec!["echo", "\'42 is the answer\'"]
        );
    }

    #[test]
    fn real_world() {
        assert_eq!(
            r#"cargo run --bin whatever -- --input="Lorem ipsum" -f"#.to_cmd(),
            vec!["cargo", "run", "--bin", "whatever",  "--", "--input=\"Lorem ipsum\"", "-f"]
        );
    }

    #[test]
    fn nested_quotes() {
        assert_eq!(
            r#"echo "lorem ipsum 'dolor' sit amet""#.to_cmd(),
            vec!["echo", "\"lorem ipsum 'dolor' sit amet\""]
        );

        assert_eq!(
            r#"echo "lorem ipsum ('dolor "doloris" septetur') sit amet""#.to_cmd(),
            vec!["echo", "\"lorem ipsum ('dolor \"doloris\" septetur') sit amet\""]
        );
    }
}
