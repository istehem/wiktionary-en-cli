use colored::ColoredString;
use colored::Colorize;
use std::fmt::Display;
use textwrap::fill;
use textwrap;

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

impl<T: Display + From<String>> Join for T {
    fn join(&self, list: Vec<T>) -> T {
        let mut res: T = T::from(String::new());
        let len: usize = list.len();
        for (i, string) in list.iter().enumerate() {
            res = T::from(format!("{}{}", res, string));
            if i < len - 1 {
                res = T::from(format!("{}{}", res, self));
            }
        }
        return res;
    }
}

impl JoinWrap for ColoredString {
    fn joinwrap(&self, list: Vec<ColoredString>, width: usize) -> ColoredString {
        let text = self.join(list);
        return wrap(&text, width);
    }
}

pub fn wrap(text: &ColoredString, width: usize) -> ColoredString {
    return fill(text, width).into();
}

pub fn indent(text: &ColoredString) -> ColoredString {
    return textwrap::indent(text, " ").into();
}

pub fn format_integer(number: usize) -> ColoredString {
    return number
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
        .yellow();
}

pub fn horizontal_line() -> ColoredString {
    return " ".repeat(LINE_WRAP_AT).strikethrough();
}
