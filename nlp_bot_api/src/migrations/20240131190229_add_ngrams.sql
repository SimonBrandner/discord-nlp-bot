CREATE TABLE ngrams (
    count INTEGER NOT NULL,
    content TEXT NOT NULL,
    time INTEGER NOT NULL,
    sender_id TEXT NOT NULL,
    container_id TEXT NOT NULL,
    UNIQUE(content, time, sender_id, container_id)
);
