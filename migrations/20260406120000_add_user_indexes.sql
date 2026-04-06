-- Add pg_trgm extension for trigram search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Add indexes to users table for search optimization
CREATE INDEX IF NOT EXISTS idx_users_username ON users USING gin (username gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_users_email ON users USING gin (email gin_trgm_ops);
