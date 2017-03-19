error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
    }
}
