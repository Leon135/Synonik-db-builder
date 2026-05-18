PRAGMA foreign_keys = ON;

CREATE TABLE words (
    id INTEGER NOT NULL PRIMARY KEY,
    word TEXT NOT NULL
);
CREATE INDEX idx_words_word ON words(word);

CREATE TABLE base_forms (
    word_id INTEGER NOT NULL REFERENCES words(id),
    base_form_id INTEGER NOT NULL REFERENCES words(id),
    PRIMARY KEY (word_id, base_form_id)
);

CREATE TABLE synonym_groups (
    group_id INTEGER NOT NULL PRIMARY KEY,
    group_meaning TEXT NOT NULL
);

CREATE TABLE word_in_group (
    word_id INTEGER NOT NULL REFERENCES words(id),
    group_id INTEGER NOT NULL REFERENCES synonym_groups(group_id),
    PRIMARY KEY (word_id, group_id)
);
CREATE INDEX idx_word_in_group_group ON word_in_group(group_id);