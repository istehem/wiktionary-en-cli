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
