-- Add migration script here
CREATE TABLE user_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    super_user BOOLEAN DEFAULT 0
);
INSERT INTO user_new (id, username, password)
SELECT id,
    username,
    password
FROM user;
DROP TABLE user;
ALTER TABLE user_new
    RENAME TO user;
INSERT INTO user (username, password, super_user)
VALUES (
        "admin",
        "$argon2id$v=19$m=19456,t=2,p=1$W37Sp/YgY00ICzwMBkby4w$HY7qML/igniHkTXawKWwzjkDJ00AzPzRljpG4eWoMNI",
        1
    );