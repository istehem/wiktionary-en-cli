use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use edit_distance::edit_distance;
use sonic_channel::*;

use utilities::language::*;

const CANNOT_OPEN_SONIC_DB_ERROR_MSG: &str = "Couldn't open sonic db, please start it";

const WIKTIONARY_COLLECTION: &str = "wiktionary";

pub struct WiktionarySearchChannel<'a> {
    language: &'a Language,
    search_channel: SearchChannel,
}

impl WiktionarySearchChannel<'_> {
    pub fn init(language: &Language) -> Result<WiktionarySearchChannel> {
        return Ok(WiktionarySearchChannel {
            language: language,
            search_channel: start_sonic_search_channel()?,
        });
    }

    pub fn query(&self, search_term: &String) -> Result<Vec<String>> {
        let objects = self.search_channel.query(
            QueryRequest::new(
                Dest::col_buc(WIKTIONARY_COLLECTION, self.language.value()),
                search_term,
            )
            .lang(to_sonic_language(self.language))
            .limit(150),
        )?;

        let mut terms: Vec<String> = Vec::new();
        for object in &objects {
            let decoded = STANDARD.decode(object)?;
            let term = String::from_utf8(decoded)?;
            terms.push(term);
        }
        return Ok(terms);
    }

    pub fn suggest(&self, search_term: &String) -> Result<Vec<String>> {
        // suggest queries for a term with spaces will restult in a server side error
        let first_word: String = search_term
            .chars()
            .take_while(|c| c != &' ' && c != &'-')
            .collect();
        let suggestions = self.search_channel.suggest(SuggestRequest::new(
            Dest::col_buc(WIKTIONARY_COLLECTION, self.language.value()),
            &first_word,
        ))?;
        return Ok(suggestions);
    }

    pub fn did_you_mean(&self, search_term: &String) -> Result<Option<String>> {
        let mut alternatives = self
            .query(search_term)
            .context(format!("could't query for term '{}'", search_term))?;
        alternatives.append(
            &mut self
                .suggest(search_term)
                .context(format!("could't suggest for term '{}'", search_term))?,
        );
        let rated_suggestions = alternatives.iter().map(|suggestion| {
            let distance = edit_distance(search_term, suggestion);
            return (
                /* an exact match, that is distance 0, is not what we are looking for */
                if distance == 0 { usize::MAX } else { distance },
                suggestion,
            );
        });
        let best_result = rated_suggestions
            .min()
            .map(|rated_result| rated_result.1.to_string());

        return Ok(best_result);
    }
}

pub struct WiktionaryIngestChannel<'a> {
    language: &'a Language,
    ingest_channel: IngestChannel,
}

impl WiktionaryIngestChannel<'_> {
    pub fn init(language: &Language) -> Result<WiktionaryIngestChannel> {
        return Ok(WiktionaryIngestChannel {
            language: language,
            ingest_channel: start_sonic_ingest_channel()?,
        });
    }

    pub fn count(&self) -> Result<u64> {
        let number_of_objects = self.ingest_channel.count(CountRequest::objects(
            WIKTIONARY_COLLECTION,
            self.language.value(),
        ))?;
        return Ok(number_of_objects as u64);
    }

    pub fn statistics(&self) -> Result<()> {
        let bucket_count = self
            .ingest_channel
            .count(CountRequest::buckets(WIKTIONARY_COLLECTION))?;
        dbg!(bucket_count);

        let object_count = self.count()?;
        dbg!(object_count);
        return Ok(());
    }

    pub fn flush(&self) -> Result<u64> {
        let flushdb_count = self.ingest_channel.flush(FlushRequest::bucket(
            WIKTIONARY_COLLECTION,
            self.language.value(),
        ))?;
        return Ok(flushdb_count as u64);
    }

    pub fn push(&self, word: &String) -> Result<()> {
        let obj = STANDARD.encode(word);
        let dest = Dest::col_buc(WIKTIONARY_COLLECTION, self.language.value()).obj(&obj);
        let push_result = self
            .ingest_channel
            .push(PushRequest::new(dest, word).lang(to_sonic_language(self.language)))?;
        return Ok(push_result);
    }
}

fn to_sonic_language(language: &Language) -> Lang {
    return match language {
        Language::EN => Lang::Eng,
        Language::DE => Lang::Deu,
        Language::SV => Lang::Swe,
        Language::FR => Lang::Fra,
        Language::ES => Lang::Spa,
    };
}

fn sonic_host() -> String {
    return env!("SONIC_HOST").to_string();
}

fn sonic_password() -> String {
    return env!("SONIC_PASSWORD").to_string();
}

fn start_sonic_search_channel() -> Result<SearchChannel> {
    let channel = SearchChannel::start(sonic_host(), sonic_password());
    return channel.map_err(|e| anyhow::Error::new(e).context(CANNOT_OPEN_SONIC_DB_ERROR_MSG));
}

fn start_sonic_ingest_channel() -> Result<IngestChannel> {
    let channel = IngestChannel::start(sonic_host(), sonic_password());
    return channel.map_err(|e| anyhow::Error::new(e).context(CANNOT_OPEN_SONIC_DB_ERROR_MSG));
}
