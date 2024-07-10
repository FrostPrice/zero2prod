-- Rename the password column to password_hash
ALTER TABLE users
    RENAME password TO password_hash;