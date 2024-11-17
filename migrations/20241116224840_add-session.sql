-- SQLITE table which holds the user sessions

CREATE TABLE session (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  expires INTEGER NOT NULL,
  created INTEGER NOT NULL,
  FOREIGN KEY (user_id) REFERENCES user (name)
);
