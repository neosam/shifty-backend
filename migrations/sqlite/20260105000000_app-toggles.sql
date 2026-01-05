-- Individual toggles
CREATE TABLE toggle (
    name TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,  -- 0 = disabled, 1 = enabled
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

-- Toggle groups
CREATE TABLE toggle_group (
    name TEXT NOT NULL PRIMARY KEY,
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

-- Junction table: which toggles belong to which groups
CREATE TABLE toggle_group_toggle (
    toggle_group_name TEXT NOT NULL,
    toggle_name TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    CONSTRAINT fk_toggle_group FOREIGN KEY (toggle_group_name) REFERENCES toggle_group(name) ON DELETE CASCADE,
    CONSTRAINT fk_toggle FOREIGN KEY (toggle_name) REFERENCES toggle(name) ON DELETE CASCADE,
    UNIQUE (toggle_group_name, toggle_name)
);

-- Add new privilege for toggle management
INSERT INTO privilege (name, update_process) VALUES ('toggle_admin', 'initial');
