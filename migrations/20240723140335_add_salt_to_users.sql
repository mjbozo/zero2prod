-- migrations/20240723140335_add_salt_to_users.sql
ALTER TABLE users ADD COLUMN salt TEXT NOT NULL;
