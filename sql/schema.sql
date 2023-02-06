CREATE TABLE topics(
    name VARCHAR(256) NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(name)
);

CREATE TABLE items(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(256) NOT NULL,
    topicname VARCHAR(256) NOT NULL,
    isrecurring INTEGER DEFAULT 0 NOT NULL,
    percentage INTEGER DEFAULT 0 NOT NULL,
    timesfinished INTEGER DEFAULT 0 NOT NULL,
    daylimit INTEGER DEFAULT 0 NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY(topicname) REFERENCES topics(name)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);
