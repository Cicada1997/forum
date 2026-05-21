CREATE TABLE IF NOT EXISTS forums (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(40) NOT NULL,
    description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS posts (
    id BIGSERIAL PRIMARY KEY,
    forum_id BIGINT NOT NULL REFERENCES forums(id),
    author_uid BIGINT NOT NULL,
    name VARCHAR(40) NOT NULL,
    sections TEXT[] NOT NULL
);

CREATE TABLE IF NOT EXISTS comments (
    id BIGSERIAL PRIMARY KEY,
    post_id BIGINT NOT NULL REFERENCES posts(id),
    author_id BIGINT NOT NULL,
    parent BIGINT REFERENCES comments(id),
    content TEXT NOT NULL
);
