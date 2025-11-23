INSERT INTO users (name, email, created_at) VALUES
    ('Alice Johnson', 'alice@example.com', datetime('now', '-7 days')),
    ('Bob Smith', 'bob@example.com', datetime('now', '-5 days')),
    ('Charlie Brown', 'charlie@example.com', datetime('now', '-3 days')),
    ('Diana Prince', 'diana@example.com', datetime('now', '-2 days')),
    ('Eve Wilson', 'eve@example.com', datetime('now', '-1 day'))
ON CONFLICT(email) DO NOTHING;

