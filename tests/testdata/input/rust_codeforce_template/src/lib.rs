pub mod pr;
// ==== BELOW IS text_io https://github.com/oli-obk/rust-si ====
// Tried to put text_io in a module, but failed for exporting macros
use std::error;
use std::fmt;
use std::str::FromStr;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Error {
    MissingMatch,
    MissingClosingBrace,
    UnexpectedValue(u8, Option<u8>),
    InvalidUtf8(Vec<u8>),
    PartialUtf8(usize, Vec<u8>),
    Parse(String, &'static str),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        use std::str::from_utf8;

        match *self {
            InvalidUtf8(ref raw) => write!(f, "input was not valid utf8: {:?}", raw),
            Parse(ref s, arg) => write!(f, "could not parse {} as target type of {}", s, arg),
            UnexpectedValue(exp, act) => write!(
                f,
                "found value {:?} not matching the pattern value {}",
                act.map(|b| b as char),
                exp as char
            ),
            PartialUtf8(n, ref raw) => write!(
                f,
                "input was only partially valid utf8: \"{}\" followed by {:?}",
                from_utf8(&raw[..n]).unwrap(),
                &raw[n..]
            ),
            MissingMatch => write!(f, "Bad read! format string: did not contain {{}}"),
            MissingClosingBrace => write!(
                f,
                "found single open curly brace at the end of the format string"
            ),
        }
    }
}

pub fn match_next(expected: u8, iter: &mut dyn Iterator<Item = u8>) -> Result<(), Error> {
    let next = iter.next();
    if next != Some(expected) {
        return Err(Error::UnexpectedValue(expected, next));
    }
    Ok(())
}

pub fn parse_capture<T>(
    name: &'static str,
    next: Option<u8>,
    iter: &mut dyn Iterator<Item = u8>,
) -> Result<T, Error>
    where
        T: FromStr,
        <T as FromStr>::Err: ::std::fmt::Debug,
{
    static WHITESPACES: &[u8] = b"\t\r\n ";
    let raw: Vec<u8> = match next {
        Some(c) => iter.take_while(|&ch| ch != c).collect(),
        None => iter
            .skip_while(|ch| WHITESPACES.contains(ch))
            .take_while(|ch| !WHITESPACES.contains(ch))
            .collect(),
    };
    match String::from_utf8(raw) {
        Ok(s) => FromStr::from_str(&s).map_err(|_| Error::Parse(s, name)),
        Err(e) => {
            let n = e.utf8_error().valid_up_to();
            let raw = e.into_bytes();
            if n == 0 {
                Err(Error::InvalidUtf8(raw))
            } else {
                Err(Error::PartialUtf8(n, raw))
            }
        }
    }
}

#[macro_export]
macro_rules! try_read(
    () => { $crate::try_read!("{}") };
    ($text:expr) => {{
        (|| -> std::result::Result<_, $crate::Error> {
            let __try_read_var__;
            $crate::try_scan!($text, __try_read_var__);
            Ok(__try_read_var__)
        })()
    }};
    ($text:expr, $input:expr) => {{
        (|| -> std::result::Result<_, $crate::Error> {
            let __try_read_var__;
            $crate::try_scan!($input => $text, __try_read_var__);
            Ok(__try_read_var__)
        })()
    }};
);

#[macro_export]
macro_rules! try_scan(
    ($pattern:expr, $($arg:expr),*) => {
        use ::std::io::Read;
        $crate::try_scan!(::std::io::stdin().bytes().map(std::result::Result::unwrap) => $pattern, $($arg),*);
        format_args!($pattern, $($arg),*);
    };
    ($input:expr => $pattern:expr, $($arg:expr),*) => {{
        $crate::try_scan!(@impl question_mark; $input => $pattern, $($arg),*)
    }};
    (@question_mark: $($e:tt)+) => {{
        ($($e)+)?
    }};
    (@unwrap: $($e:tt)+) => {{
        ($($e)+).unwrap()
    }};
    (@impl $action:tt; $input:expr => $pattern:expr, $($arg:expr),*) => {{
        #![allow(clippy::try_err)]
        use $crate::{Error, match_next, parse_capture};

        // typesafe macros :)
        let pattern: &'static str = $pattern;
        let stdin: &mut Iterator<Item = u8> = &mut ($input);

        let mut pattern = pattern.bytes();

        $(
            $arg = loop {
                match $crate::try_scan!(@$action: pattern.next().ok_or(Error::MissingMatch)) {
                    b'{' => match $crate::try_scan!(@$action: pattern.next().ok_or(Error::MissingClosingBrace)) {
                        b'{' => $crate::try_scan!(@$action: match_next(b'{', stdin)),
                        b'}' => break $crate::try_scan!(@$action: parse_capture(stringify!($arg), pattern.next(), stdin)),
                        _ => return $crate::try_scan!(@$action: Err(Error::MissingClosingBrace)),
                    },
                    c => $crate::try_scan!(@$action: match_next(c, stdin)),
                }
            };
        )*

        for c in pattern {
            $crate::try_scan!(@$action: match_next(c, stdin))
        }

        format_args!($pattern, $($arg),*);
    }};
);

/// All text input is handled through this macro
#[macro_export]
macro_rules! read(
    ($($arg:tt)*) => {
        $crate::try_read!($($arg)*).unwrap()
    };
);

/// This macro allows to pass several variables so multiple values can be read
#[macro_export]
macro_rules! scan(
    ($text:expr, $($arg:expr),*) => {
        use ::std::io::Read;
        $crate::scan!(::std::io::stdin().bytes().map(std::result::Result::unwrap) => $text, $($arg),*);
        format_args!($text, $($arg),*);
    };
    ($input:expr => $pattern:expr, $($arg:expr),*) => {{
        $crate::try_scan!(@impl unwrap; $input => $pattern, $($arg),*)
    }};
);
