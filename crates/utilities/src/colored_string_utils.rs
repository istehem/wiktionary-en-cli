use colored::ColoredString;
use colored::Colorize;
use std::fmt::Display;
use textwrap;
use textwrap::fill;

pub const NEWLINE: &str = "\n";
pub const LINE_WRAP_AT: usize = 80;

pub trait Join {
    fn join(&self, list: Vec<Self>) -> Self
    where
        Self: Sized;
}

pub trait JoinWrap {
    fn joinwrap(&self, list: Vec<Self>, width: usize) -> Self
    where
        Self: Sized;
}

impl<T> Join for T
where
    T: Display + From<String>,
{
    fn join(&self, list: Vec<T>) -> T {
        let mut res: T = T::from(String::new());
        let len: usize = list.len();
        for (i, string) in list.iter().enumerate() {
            res = T::from(format!("{}{}", res, string));
            if i < len - 1 {
                res = T::from(format!("{}{}", res, self));
            }
        }
        res
    }
}

impl JoinWrap for ColoredString {
    fn joinwrap(&self, list: Vec<ColoredString>, width: usize) -> ColoredString {
        let text = self.join(list);
        wrap(&text, width)
    }
}

pub fn wrap(text: &ColoredString, width: usize) -> ColoredString {
    fill(text, width).into()
}

pub fn indent(text: &ColoredString) -> ColoredString {
    textwrap::indent(text, " ").into()
}

pub fn format_integer(number: usize) -> ColoredString {
    number
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
        .yellow()
}

pub fn horizontal_line() -> ColoredString {
    " ".repeat(LINE_WRAP_AT).strikethrough()
}
