-- migrations/20240723132615_rename_password_column.sql
ALTER TABLE users RENAME password TO password_hash;
