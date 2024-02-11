CREATE TABLE new_containers (
    container_id TEXT NOT NULL,
    container_parent_id TEXT NOT NULL,
    UNIQUE(container_id, container_parent_id)
);

INSERT INTO
    new_containers (container_id, container_parent_id)
SELECT
    DISTINCT container_id,
    container_parent_id
FROM
    containers;

DROP TABLE containers;

ALTER TABLE
    new_containers RENAME TO containers;
