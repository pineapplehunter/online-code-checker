-- Add migration script here
CREATE TABLE solutions (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    content TEXT,
    status TEXT,
    userid INTEGER NOT NULL,
    FOREIGN KEY (userid) REFERENCES user(id)
)