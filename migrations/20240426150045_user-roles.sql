-- Add migration script here
CREATE TABLE user (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
CREATE TRIGGER user_update_timestamp
  AFTER UPDATE ON user
  BEGIN
    UPDATE user SET update_timestamp = DATETIME('now') WHERE rowid = old.rowid;
  END;

CREATE TABLE role (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
CREATE TRIGGER role_update_timestamp
  AFTER UPDATE ON role
  BEGIN
    UPDATE role SET update_timestamp = DATETIME('now') WHERE rowid = old.rowid;
  END;

CREATE TABLE privilege (
    name TEXT NOT NULL PRIMARY KEY,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);
CREATE TRIGGER privilege_update_timestamp
  AFTER UPDATE ON privilege
  BEGIN
    UPDATE privilege SET update_timestamp = DATETIME('now') WHERE rowid = old.rowid;
  END;

CREATE TABLE user_role (
    user_name TEXT NOT NULL,
    role_name TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_name) REFERENCES user(name) ON DELETE CASCADE,
    CONSTRAINT fk_role FOREIGN KEY (role_name) REFERENCES role(name) ON DELETE CASCADE
    UNIQUE (user_name, role_name)
);
CREATE TRIGGER user_role_update_timestamp
  AFTER UPDATE ON user_role
  BEGIN
    UPDATE user_role SET update_timestamp = DATETIME('now') WHERE rowid = old.rowid;
  END;

CREATE TABLE role_privilege (
    role_name TEXT NOT NULL,
    privilege_name TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    CONSTRAINT fk_group FOREIGN KEY (role_name) REFERENCES role(name) ON DELETE CASCADE,
    CONSTRAINT fk_privilege FOREIGN KEY (privilege_name) REFERENCES privilege(name) ON DELETE CASCADE
    UNIQUE (role_name, privilege_name)
);
CREATE TRIGGER role_privilege_update_timestamp
  AFTER UPDATE ON role_privilege
  BEGIN
    UPDATE role_privilege SET update_timestamp = DATETIME('now') WHERE rowid = old.rowid;
  END;
