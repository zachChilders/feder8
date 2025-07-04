-- Create actors table
CREATE TABLE IF NOT EXISTS actors (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    summary TEXT,
    public_key_pem TEXT NOT NULL,
    private_key_pem TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Create index for username lookups
CREATE INDEX IF NOT EXISTS idx_actors_username ON actors(username);

-- Create index for created_at for sorting
CREATE INDEX IF NOT EXISTS idx_actors_created_at ON actors(created_at);