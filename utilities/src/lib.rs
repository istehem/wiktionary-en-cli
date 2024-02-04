pub mod colored_string_utils {
    use colored::Colorize;
    use colored::ColoredString;
    use textwrap::fill;

    pub const NEWLINE: &str = "\n";
    pub const LINE_WRAP_AT: usize = 80;

    pub trait Join {
        fn join(&self, list : Vec<Self>) -> Self where Self: Sized;
        fn joinwrap(&self, list : Vec<Self>, width : usize) -> Self where Self: Sized;
    }

    impl Join for ColoredString {
        fn join(&self, list : Vec<ColoredString>) -> ColoredString {
            let mut res : ColoredString = "".normal();
            let len : usize = list.len();
            for (i, string) in list.iter().enumerate() {
                res = format!("{}{}", res, string).normal();
                if i < len - 1 {
                    res = format!("{}{}", res, self).normal();
                }
            }
            return res.clone();
        }

        fn joinwrap(&self, list : Vec<ColoredString>, width : usize) -> ColoredString {
            let text = self.join(list);
            return fill(&text, width).normal();
        }
    }

    pub fn format_integer(number: usize) -> ColoredString {
        return number.to_string()
            .as_bytes()
            .rchunks(3)
            .rev()
            .map(std::str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap()
            .join(",").yellow();
    }

    pub fn horizontal_line() -> ColoredString {
        return " ".repeat(LINE_WRAP_AT).strikethrough();
    }
}

pub mod file_utils {
    use std::io::{BufReader};
    use anyhow::{Result, bail};
    use std::path::{Path};
    use std::fs::File;

    pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
        let file_buffer_result =  File::open(path)
            .map(|f| BufReader::new(f))
            .map_err(|err| anyhow::Error::new(err));
        match file_buffer_result {
            ok@Ok(_) => return ok,
            _        => bail!("No such DB file: '{}'", path.display())

        }
    }
}
