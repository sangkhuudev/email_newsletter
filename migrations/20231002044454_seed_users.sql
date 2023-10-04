-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
'ddf8994f-d522-4659-8d02-c1d479057be6',
'admin',
'$argon2id$v=19$m=15000,t=2,p=1$gEF2mZ5NpKoaYy+LDrFMQw$s3OsK+Pu79U6NvQThm1arXjPdr5p3beFJWp9LjKTM5A'
);