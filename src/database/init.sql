-- USE qpui_db;

CREATE TYPE category AS ENUM ('Informatique', 'Sciences', 'Internet', 'Histoire', 'Geographie', 'Classique', 'Moderne', 'Sport', 'Autre');;

CREATE TABLE questions (
    id SERIAL PRIMARY KEY,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    category category NOT NULL,
    notes TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE
);;

CREATE TABLE tracks (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    notes TEXT,
    is_public BOOLEAN NOT NULL
);;

CREATE TABLE question_tracks (
    id SERIAL PRIMARY KEY,
    question_id INTEGER NOT NULL REFERENCES questions(id) ON DELETE CASCADE,
    track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    position INTEGER NOT NULL
);;