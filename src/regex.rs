#[macro_export]
macro_rules! lazy_regex {
    { $($e:expr)* } => {{
        use ::std::sync::LazyLock;
        use ::regex_lite::Regex;

        static __RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(concat!($($e),*)).unwrap()
        });

        &*__RE
    }};
}

#[macro_export]
macro_rules! regex {
    { $($e:expr)* } => {
        concat!($($e),*)
    };
}

#[macro_export]
macro_rules! re_capture {
    { [$name:ident] $($e:expr)* } => {
        concat!(r"(?<", stringify!($name), ">", $($e),*, r")")
    };
}

#[macro_export]
macro_rules! re_optional {
    { $($e:expr)* } => {
        concat!(r"(?:", $($e),*, r")?")
    };
}

#[macro_export]
macro_rules! re_repeat {
    { [+] $($e:expr)* } => {
        concat!(r"(?:", $($e),*, r")+")
    };
    { [*] $($e:expr)* } => {
        concat!(r"(?:", $($e),*, r")*")
    };
    { [$count:expr] $($e:expr)* } => {
        concat!(r"(?:", $($e),*, r"){", $count, "}")
    };
}

#[macro_export]
macro_rules! re_seplist {
    { [$sep:expr] $($e:expr)* } => {
        concat!($($e),*, "(?:", $sep, $($e),*, ")*")
    };
}
