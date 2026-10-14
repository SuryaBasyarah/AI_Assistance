INSERT INTO memories(content, importance, active, created_at, updated_at)
SELECT
    'Haruya''s admin name is Haruya Sariel.',
    1.0,
    1,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE NOT EXISTS (
    SELECT 1 FROM memories
    WHERE lower(content) = lower('Haruya''s admin name is Haruya Sariel.')
);
