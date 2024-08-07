-- 20240805060145_seed_user.sql

INSERT INTO users (user_id, username, password_hash)
VALUES (
    '85fbb63b-2a00-48f8-8e73-8b5e339da33c',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$YPkfny0tcIJyiEhJKtdlWA$mwtPk3rLajPcqsweLieIJKqQqCzQ+3/3B1WUe98Xsbw'
);
