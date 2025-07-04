-- Create follows table
CREATE TABLE IF NOT EXISTS follows (
    id TEXT PRIMARY KEY,
    follower_id TEXT NOT NULL,
    following_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (follower_id) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (following_id) REFERENCES actors(id) ON DELETE CASCADE,
    UNIQUE(follower_id, following_id)
);

-- Create index for follower_id lookups (following list)
CREATE INDEX IF NOT EXISTS idx_follows_follower_id ON follows(follower_id);

-- Create index for following_id lookups (followers list)
CREATE INDEX IF NOT EXISTS idx_follows_following_id ON follows(following_id);

-- Create index for status filtering
CREATE INDEX IF NOT EXISTS idx_follows_status ON follows(status);

-- Create index for created_at for sorting
CREATE INDEX IF NOT EXISTS idx_follows_created_at ON follows(created_at DESC);