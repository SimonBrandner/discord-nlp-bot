use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};

pub fn ngram_time_to_date(time: i64) -> NaiveDate {
    let naive_datetime = NaiveDateTime::from_timestamp_opt(time, 0)
        .expect("You are reaching a little to far into the future!");

    #[allow(clippy::unwrap_used)]
    let datetime: DateTime<FixedOffset> =
        DateTime::from_naive_utc_and_offset(naive_datetime, FixedOffset::east_opt(0).unwrap());

    datetime.date_naive()
}
