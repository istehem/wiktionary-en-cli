pub mod colored_string_utils {
    use colored::ColoredString;
    use colored::Colorize;
    use textwrap::fill;

    pub const NEWLINE: &str = "\n";
    pub const LINE_WRAP_AT: usize = 80;

    pub trait Join {
        fn join(&self, list: Vec<Self>) -> Self
        where
            Self: Sized;
        fn joinwrap(&self, list: Vec<Self>, width: usize) -> Self
        where
            Self: Sized;
    }

    impl Join for ColoredString {
        fn join(&self, list: Vec<ColoredString>) -> ColoredString {
            let mut res: ColoredString = "".normal();
            let len: usize = list.len();
            for (i, string) in list.iter().enumerate() {
                res = format!("{}{}", res, string).normal();
                if i < len - 1 {
                    res = format!("{}{}", res, self).normal();
                }
            }
            return res;
        }

        fn joinwrap(&self, list: Vec<ColoredString>, width: usize) -> ColoredString {
            let text = self.join(list);
            return fill(&text, width).normal();
        }
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
}

pub mod file_utils {
    use anyhow::{bail, Result};
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
        let file_buffer_result = File::open(path)
            .map(|f| BufReader::new(f))
            .map_err(|err| anyhow::Error::new(err));
        match file_buffer_result {
            ok @ Ok(_) => return ok,
            _ => bail!("No such DB file: '{}'", path.display()),
        }
    }
}
pub mod cache_utils {
    use crate::anyhow_serde;
    use anyhow::{bail, Context, Result};

    pub fn write_db_entry_to_cache<T: serde::Serialize>(
        path: &String,
        term: &String,
        value: &T,
    ) -> Result<()> {
        // this directory will be created if it does not exist

        let db = sled::open(path)
            .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));

        let json = anyhow_serde::to_string(value).context(format!("cannot serialize entry"));

        return json.and_then(|json| {
            db.and_then(|db| {
                db.insert(term, json.as_bytes())
                    .map_err(|err| anyhow::Error::new(err))
                    .map(|_| return)
            })
        });
    }

    pub fn get_cached_db_entry<T: for<'a> serde::Deserialize<'a>>(
        path: &String,
        term: &String,
    ) -> Result<Option<T>> {
        let db = sled::open(path)
            .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));

        match db.and_then(|db| db.get(term).map_err(|err| anyhow::Error::new(err))) {
            Ok(Some(b)) => {
                return String::from_utf8((&b).to_vec())
                    .map_err(|err| anyhow::Error::new(err))
                    .and_then(|s| anyhow_serde::from_str(&s))
            }
            Ok(_) => return Ok(None),
            Err(err) => bail!(err),
        };
    }

    pub fn get_number_of_entries(path: &String) -> Result<usize> {
        let db = sled::open(path)
            .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));
        return db.map(|db| db.iter().count());
    }

    pub fn get_size_on_disk(path: &String) -> Result<u64> {
        let db = sled::open(path)
            .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));
        return db.and_then(|db| {
            db.size_on_disk()
                .map_err(|err| anyhow::Error::new(err))
                .context(format!("couldn't determine disk space for db: {}", path))
        });
    }
}

pub mod language {

    use self::Language::*;

    #[derive(Copy, Clone)]
    pub enum Language {
        EN,
        DE,
        FR,
        ES,
        SV,
    }

    impl Language {
        pub fn value(&self) -> String {
            match self {
                EN => "en".to_string(),
                DE => "de".to_string(),
                FR => "fr".to_string(),
                ES => "es".to_string(),
                SV => "sv".to_string(),
            }
        }
        pub fn iterator() -> impl Iterator<Item = Language> {
            [EN, DE, FR, ES, SV].iter().copied()
        }
        pub fn as_strings() -> Vec<String> {
            return Language::iterator().map(|lang| lang.value()).collect();
        }
        pub fn from_string(language: &String) -> Option<Language> {
            return Language::iterator().find(|l| &l.value() == language);
        }
    }

    impl Default for Language {
        fn default() -> Self {
            EN
        }
    }
}

pub mod anyhow_serde {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    pub fn from_str<T: for<'a> Deserialize<'a>>(line: &String) -> Result<T> {
        return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
    }

    pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
        return serde_json::to_string(value).map_err(|err| anyhow::Error::new(err));
    }
}
