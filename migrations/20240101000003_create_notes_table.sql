-- Create notes table
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    attributed_to TEXT NOT NULL,
    content TEXT NOT NULL,
    to_recipients TEXT NOT NULL, -- JSON array
    cc_recipients TEXT NOT NULL, -- JSON array
    published DATETIME NOT NULL,
    in_reply_to TEXT,
    tags TEXT NOT NULL, -- JSON array
    created_at DATETIME NOT NULL,
    FOREIGN KEY (attributed_to) REFERENCES actors(id) ON DELETE CASCADE,
    FOREIGN KEY (in_reply_to) REFERENCES notes(id) ON DELETE SET NULL
);

-- Create index for attributed_to lookups
CREATE INDEX IF NOT EXISTS idx_notes_attributed_to ON notes(attributed_to);

-- Create index for published date for sorting
CREATE INDEX IF NOT EXISTS idx_notes_published ON notes(published DESC);

-- Create index for in_reply_to for threading
CREATE INDEX IF NOT EXISTS idx_notes_in_reply_to ON notes(in_reply_to);

-- Create index for to_recipients
CREATE INDEX IF NOT EXISTS idx_notes_to_recipients ON notes(to_recipients);

-- Create index for cc_recipients
CREATE INDEX IF NOT EXISTS idx_notes_cc_recipients ON notes(cc_recipients);