CREATE TABLE sessions (
    id uuid PRIMARY KEY,
    user_id uuid UNIQUE NOT NULL REFERENCES users,
    created_at timestamp NOT NULL DEFAULT NOW()
)