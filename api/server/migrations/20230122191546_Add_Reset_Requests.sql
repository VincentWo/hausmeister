CREATE TABLE password_reset_requests (
    id uuid PRIMARY KEY,
    user_id uuid UNIQUE NOT NULL REFERENCES users,
    created_at timestamp NOT NULL DEFAULT NOW()
)