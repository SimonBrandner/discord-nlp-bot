CREATE TABLE messages (
    message_id TEXT NOT NULL,
    container_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    unix_timestamp INT NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE containers (
    container_id TEXT NOT NULL,
    container_parent_id TEXT
);
