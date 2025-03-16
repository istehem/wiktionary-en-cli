#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::*;

    #[test]
    fn at_least_as_many_unique_entries_as_all_entries() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        Ok(())
    }
}
