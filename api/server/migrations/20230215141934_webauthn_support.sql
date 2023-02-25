CREATE TABLE webauthn_passkeys (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL,
    key_data JSON NOT NULL,
    CONSTRAINT fk_user_id
        FOREIGN KEY (user_id)
            REFERENCES users
)
