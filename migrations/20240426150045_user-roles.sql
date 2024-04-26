-- Add migration script here
CREATE TABLE user (
    id BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE role (
    id BLOB(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE privilege (
    name TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE user_role (
    user_id BLOB(16) NOT NULL,
    role_id BLOB(16) NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    CONSTRAINT fk_role FOREIGN KEY (role_id) REFERENCES role(id) ON DELETE CASCADE

);

CREATE TABLE role_privilege (
    role_id BLOB(16) NOT NULL,
    privilege_name TEXT NOT NULL,
    CONSTRAINT fk_group FOREIGN KEY (role_id) REFERENCES role(id) ON DELETE CASCADE,
    CONSTRAINT fk_privilege FOREIGN KEY (privilege_name) REFERENCES privilege(name) ON DELETE CASCADE
);
