DROP TABLE IF EXISTS ngrams;

CREATE TABLE ngrams (
    count INTEGER NOT NULL,
    content TEXT NOT NULL,
    length INTEGER NOT NULL,
    time INTEGER NOT NULL,
    sender_id TEXT NOT NULL,
    container_id TEXT NOT NULL,
    UNIQUE(content, length, time, sender_id, container_id)
);

UPDATE
    entries
SET
    ngrams_cached = false;
