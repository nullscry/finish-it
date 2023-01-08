CREATE TABLE events(
    name VARCHAR(256) NOT NULL,
    eventgroup VARCHAR(256) DEFAULT "OTHER" NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(name)
);

CREATE TABLE instances(
    instanceid INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(256) NOT NULL,
    eventtype VARCHAR(256) NOT NULL,
    isrecurring INTEGER DEFAULT 0 NOT NULL,
    isfinished INTEGER DEFAULT 0 NOT NULL,
    percentage REAL DEFAULT 0.0 NOT NULL,
    timesfinished INTEGER DEFAULT 0 NOT NULL,
    daylimit INTEGER DEFAULT 0 NOT NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY(eventtype) REFERENCES event(name)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

-- CREATE TABLE following(
--   username1 VARCHAR(20) NOT NULL,
--   username2 VARCHAR(20) NOT NULL,
--   created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
--   PRIMARY KEY(username1, username2),
--   FOREIGN KEY(username1) REFERENCES users(username)
--     ON DELETE CASCADE
--     ON UPDATE CASCADE,
--   FOREIGN KEY(username2) REFERENCES users(username)
--     ON DELETE CASCADE
--     ON UPDATE CASCADE
-- );

-- CREATE TABLE comments(
--   commentid INTEGER PRIMARY KEY AUTOINCREMENT,
--   owner VARCHAR(20) NOT NULL,
--   postid INT NOT NULL,
--   text VARCHAR(1024) NOT NULL,
--   created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
--   FOREIGN KEY(owner) REFERENCES users(username)
--     ON DELETE CASCADE
--     ON UPDATE CASCADE,
--   FOREIGN KEY(postid) REFERENCES posts(postid)
--     ON DELETE CASCADE
--     ON UPDATE CASCADE
-- );

-- CREATE TABLE likes(
--   owner VARCHAR(20) NOT NULL,
--   postid INT NOT NULL,
--   created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
--   PRIMARY KEY(owner, postid),
--   FOREIGN KEY(postid) REFERENCES posts(postid)
--     ON DELETE CASCADE
--     ON UPDATE CASCADE
-- );
