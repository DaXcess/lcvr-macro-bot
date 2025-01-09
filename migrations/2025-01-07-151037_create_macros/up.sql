-- Your SQL goes here
CREATE TABLE
    IF NOT EXISTS macro (
        id INTEGER PRIMARY KEY NOT NULL,
        name VARCHAR(32) NOT NULL UNIQUE,
        description VARCHAR(250) NOT NULL,
        content TEXT NOT NULL
    );

CREATE TABLE
    IF NOT EXISTS attachment (
        id INTEGER PRIMARY KEY NOT NULL,
        macro_id INTEGER NOT NULL,
        link VARCHAR(250) NOT NULL,
        FOREIGN KEY (macro_id) REFERENCES macro (id) ON DELETE CASCADE
    )