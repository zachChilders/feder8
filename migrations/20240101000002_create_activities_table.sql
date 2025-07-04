-- Create activities table
CREATE TABLE IF NOT EXISTS activities (
    id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    object TEXT NOT NULL, -- JSON string
    to_recipients TEXT NOT NULL, -- JSON array
    cc_recipients TEXT NOT NULL, -- JSON array
    published DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (actor_id) REFERENCES actors(id) ON DELETE CASCADE
);

-- Create index for actor_id lookups
CREATE INDEX IF NOT EXISTS idx_activities_actor_id ON activities(actor_id);

-- Create index for published date for sorting
CREATE INDEX IF NOT EXISTS idx_activities_published ON activities(published DESC);

-- Create index for activity_type filtering
CREATE INDEX IF NOT EXISTS idx_activities_type ON activities(activity_type);

-- Create index for to_recipients (for inbox queries)
CREATE INDEX IF NOT EXISTS idx_activities_to_recipients ON activities(to_recipients);

-- Create index for cc_recipients (for inbox queries)
CREATE INDEX IF NOT EXISTS idx_activities_cc_recipients ON activities(cc_recipients);