use super::ngram::{
    get_ngram_time, get_ngrams_in_word_list, Ngram, ALLOWED_NGRAM_CHARACTERS_REGEX,
    MAX_NGRAM_LENGTH,
};

#[derive(Debug)]
pub struct Entry {
    pub entry_id: String,
    pub container_id: String,
    pub sender_id: String,
    pub unix_timestamp: i64,
    pub content: String,
}

impl Entry {
    pub fn get_ngrams(&self) -> Vec<Ngram> {
        let lower_case_content = self.content.to_lowercase();
        let words: Vec<String> = ALLOWED_NGRAM_CHARACTERS_REGEX
            .find_iter(&lower_case_content)
            .map(|mat| mat.as_str().to_string())
            .collect();

        get_ngrams_in_word_list(words.as_slice(), MAX_NGRAM_LENGTH)
            .iter()
            .map(|words| Ngram {
                content: words.join(" "),
                #[allow(clippy::cast_possible_truncation)]
                length: words.len() as u32,
                time: get_ngram_time(self.unix_timestamp),
                container_id: self.container_id.clone(),
                sender_id: self.sender_id.clone(),
            })
            .collect()
    }

    pub fn get_ngrams_from_entries_slice(entries: &[Self]) -> Vec<Ngram> {
        let mut ngrams = Vec::new();
        for entry in entries {
            ngrams.extend(entry.get_ngrams());
        }
        ngrams
    }
}
