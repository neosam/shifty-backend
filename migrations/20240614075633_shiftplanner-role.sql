-- Add migration script here

INSERT INTO role (name, update_process) VALUES ('shiftplanner', 'update-2024-06-14');

INSERT INTO privilege (name, update_process) VALUES ('shiftplanner', 'update-2024-06-14');

INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('shiftplanner', 'shiftplanner', 'update-2024-06-14');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('admin', 'shiftplanner', 'update-2024-06-14');