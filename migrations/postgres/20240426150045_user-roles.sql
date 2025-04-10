-- Example PostgreSQL migration for user roles
-- This is just a sample to demonstrate the structure

CREATE TABLE "user" (
    name TEXT PRIMARY KEY,
    update_timestamp TIMESTAMP,
    update_process TEXT NOT NULL,
    update_version UUID NOT NULL
);

CREATE TABLE role (
    name TEXT PRIMARY KEY,
    update_timestamp TIMESTAMP,
    update_process TEXT NOT NULL,
    update_version UUID NOT NULL
);

CREATE TABLE privilege (
    name TEXT PRIMARY KEY,
    update_timestamp TIMESTAMP,
    update_process TEXT NOT NULL,
    update_version UUID NOT NULL
);

CREATE TABLE user_role (
    user_name TEXT REFERENCES "user"(name) ON DELETE CASCADE,
    role_name TEXT REFERENCES role(name) ON DELETE CASCADE,
    update_timestamp TIMESTAMP,
    update_process TEXT NOT NULL,
    update_version UUID NOT NULL,
    PRIMARY KEY (user_name, role_name)
);

CREATE TABLE role_privilege (
    role_name TEXT REFERENCES role(name) ON DELETE CASCADE,
    privilege_name TEXT REFERENCES privilege(name) ON DELETE CASCADE,
    update_timestamp TIMESTAMP,
    update_process TEXT NOT NULL,
    update_version UUID NOT NULL,
    PRIMARY KEY (role_name, privilege_name)
);

-- Insert default privileges
INSERT INTO privilege (name, update_process, update_version)
VALUES ('admin', 'initial-setup', '00000000-0000-0000-0000-000000000000');

INSERT INTO privilege (name, update_process, update_version)
VALUES ('user', 'initial-setup', '00000000-0000-0000-0000-000000000000');

-- Insert default roles
INSERT INTO role (name, update_process, update_version)
VALUES ('admin', 'initial-setup', '00000000-0000-0000-0000-000000000000');

INSERT INTO role (name, update_process, update_version)
VALUES ('user', 'initial-setup', '00000000-0000-0000-0000-000000000000');

-- Assign privileges to roles
INSERT INTO role_privilege (role_name, privilege_name, update_process, update_version)
VALUES ('admin', 'admin', 'initial-setup', '00000000-0000-0000-0000-000000000000');

INSERT INTO role_privilege (role_name, privilege_name, update_process, update_version)
VALUES ('admin', 'user', 'initial-setup', '00000000-0000-0000-0000-000000000000');

INSERT INTO role_privilege (role_name, privilege_name, update_process, update_version)
VALUES ('user', 'user', 'initial-setup', '00000000-0000-0000-0000-000000000000');
