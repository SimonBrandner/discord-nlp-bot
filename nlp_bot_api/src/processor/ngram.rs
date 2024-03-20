use lazy_static::lazy_static;
use regex::Regex;

// TODO: Should we just split on whitespace?
/// We only allow English, Czech, Spanish and German characters though
/// contributions to handle more complex situations are welcome!
const ALLOWED_NGRAM_CHARACTERS: &str = r"[a-z0-9ěščřžýáíéñüßöäňůúó']+";
const SECONDS_IN_WEEK: i64 = 7 * 24 * 60 * 60;

pub const MAX_NGRAM_LENGTH: u8 = 5;
lazy_static! {
    pub static ref ALLOWED_NGRAM_CHARACTERS_REGEX: Regex = #[allow(clippy::unwrap_used)] // A failing regex would be discovered on first run
    Regex::new(ALLOWED_NGRAM_CHARACTERS)
        .unwrap();
}

fn get_ngram(words: &[String], ngram_length: u8, start_index: usize) -> Option<&[String]> {
    if start_index + ngram_length as usize > words.len() {
        return None;
    }

    Some(&words[start_index..(start_index + ngram_length as usize)])
}

pub fn get_ngrams_in_word_list(words: &[String], max_ngram_length: u8) -> Vec<&[String]> {
    let mut ngrams: Vec<&[String]> = Vec::new();

    for index in 1..=words.len() {
        for ngram_length in 1..=max_ngram_length {
            match get_ngram(words, ngram_length, index) {
                Some(ngram) => {
                    ngrams.push(ngram);
                }
                None => break,
            }
        }
    }

    ngrams
}

pub const fn get_ngram_time(time: i64) -> i64 {
    time - (time % SECONDS_IN_WEEK)
}

pub fn fill_gaps(ngrams: &mut Vec<NgramsForByContentCommand>) {
    if ngrams.len() < 2 {
        return;
    }

    for i in 1..(ngrams.len() - 1) {
        let diff = ngrams[i].time - ngrams[i - 1].time;

        let n = i;
        if diff > 2 * SECONDS_IN_WEEK {
            ngrams.splice(
                n..n,
                [
                    NgramsForByContentCommand {
                        time: ngrams[i - 1].time + SECONDS_IN_WEEK,
                        count: 0,
                    },
                    NgramsForByContentCommand {
                        time: ngrams[i].time - SECONDS_IN_WEEK,
                        count: 0,
                    },
                ],
            );
        } else if diff > SECONDS_IN_WEEK {
            ngrams.splice(
                n..n,
                [NgramsForByContentCommand {
                    time: ngrams[i - 1].time + SECONDS_IN_WEEK,
                    count: 0,
                }],
            );
        }
    }
}

pub struct NgramForStore {
    pub content: String,
    pub length: u32,
    pub time: i64,
    pub container_id: String,
    pub sender_id: String,
}

pub struct NgramForByCountCommand {
    pub content: String,
    pub count: u32,
}

#[derive(Clone)]
pub struct NgramsForByContentCommand {
    pub count: u32,
    pub time: i64,
}
