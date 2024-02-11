CREATE TABLE new_entries (
    entry_id TEXT NOT NULL UNIQUE PRIMARY KEY,
    container_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    unix_timestamp INT NOT NULL,
    content TEXT NOT NULL,
    ngrams_cached BOOLEAN NOT NULL DEFAULT false
);

INSERT INTO
    new_entries (
        entry_id,
        container_id,
        sender_id,
        unix_timestamp,
        content,
        ngrams_cached
    )
SELECT
    DISTINCT entry_id,
    container_id,
    sender_id,
    unix_timestamp,
    content,
    ngrams_cached
FROM
    entries;

DROP TABLE entries;

ALTER TABLE
    new_entries RENAME TO entries;
