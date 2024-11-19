
INSERT INTO privilege (name, update_process) VALUES ('shiftplan.edit', 'add-role-shiftplan-edit');
INSERT INTO role (name, update_process) VALUES ('shiftplan.edit', 'add-role-shiftplan-edit');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('shiftplan.edit', 'shiftplan.edit', 'add-role-shiftplan-edit');
INSERT INTO role_privilege (role_name, privilege_name, update_process) VALUES ('admin', 'shiftplan.edit', 'add-role-shiftplan-edit');