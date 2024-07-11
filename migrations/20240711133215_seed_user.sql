-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
        'ead310dc-5378-4071-83d7-544c1ca1b6a4',
        'admin',
        '$argon2id$v=19$m=15000,t=2,p=1$27H8vGX8CucyylwAd9EcQw$TdpvlzF1CDbdE9sYyXPJTwpwJCoBOIsU30s+dCNtyi0'
    )