-- Add parent_id column to comments table for reply functionality
ALTER TABLE comments
ADD COLUMN IF NOT EXISTS parent_id bigint REFERENCES comments(id) ON DELETE CASCADE;