use chrono::{DateTime, NaiveDate};

pub fn ngram_time_to_date(time: i64) -> NaiveDate {
    DateTime::from_timestamp(time, 0)
        .expect("You are reaching a little to far into the future!")
        .date_naive()
}
