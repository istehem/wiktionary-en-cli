use crate::dictionary_entry::DictionaryEntry;
use colored::Colorize;
use indoc::formatdoc;
use std::fmt;

#[derive(Clone)]
pub struct DidYouMean {
    pub searched_for: String,
    pub suggestion: String,
}

impl fmt::Display for DidYouMean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            &did_you_mean_banner(&self.searched_for, &self.suggestion)
        )
    }
}

#[derive(Clone)]
pub struct DictionaryResult {
    pub word: String,
    pub did_you_mean: Option<DidYouMean>,
    pub hits: Vec<DictionaryEntry>,
}

impl fmt::Display for DictionaryResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(did_you_mean) = &self.did_you_mean {
            writeln!(
                f,
                "{}",
                did_you_mean_banner(&did_you_mean.searched_for, &did_you_mean.suggestion)
            )?;
        }
        for hit in &self.hits {
            writeln!(f, "{}", &hit)?;
        }
        Ok(())
    }
}

fn did_you_mean_banner(search_term: &str, partial_match: &str) -> String {
    formatdoc!(
        "
        No result for {}.
        Did you mean  {}?
        ",
        search_term.red(),
        partial_match.yellow()
    )
}
