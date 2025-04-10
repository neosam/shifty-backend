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
CREATE TRIGGER user_insert_timestamp
  AFTER INSERT ON user
  BEGIN
    UPDATE user SET update_timestamp = DATETIME('now') WHERE rowid = new.rowid;
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
CREATE TRIGGER role_insert_timestamp
  AFTER INSERT ON role
  BEGIN
    UPDATE role SET update_timestamp = DATETIME('now') WHERE rowid = new.rowid;
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
CREATE TRIGGER privilege_insert_timestamp
  AFTER INSERT ON privilege
  BEGIN
    UPDATE privilege SET update_timestamp = DATETIME('now') WHERE rowid = new.rowid;
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
CREATE TRIGGER user_role_insert_timestamp
  AFTER INSERT ON user_role
  BEGIN
    UPDATE user_role SET update_timestamp = DATETIME('now') WHERE rowid = new.rowid;
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
CREATE TRIGGER role_privilege_insert_timestamp
  AFTER INSERT ON role_privilege
  BEGIN
    UPDATE role_privilege SET update_timestamp = DATETIME('now') WHERE rowid = new.rowid;
  END;

CREATE VIEW V_UUID_V4 as
SELECT 
  lower(
    hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-' || '4' || 
    substr(hex( randomblob(2)), 2) || '-' || 
    substr('AB89', 1 + (abs(random()) % 4) , 1)  ||
    substr(hex(randomblob(2)), 2) || '-' || 
    hex(randomblob(6))
  ) UUID;

INSERT INTO role (name, update_process) VALUES ('admin', 'initial');
INSERT INTO role (name, update_process) VALUES ('sales', 'initial');
INSERT INTO role (name, update_process) VALUES ('hr', 'initial');

INSERT INTO privilege (name, update_process) VALUES ('admin', 'initial');
INSERT INTO privilege (name, update_process) VALUES ('sales', 'initial');
INSERT INTO privilege (name, update_process) VALUES ('hr', 'initial');

INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('sales', 'sales', 'initial');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('hr', 'hr', 'initial');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('admin', 'admin', 'initial');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('admin', 'sales', 'initial');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('admin', 'hr', 'initial');
