-- Fix comments table structure to support guest comments and replies
ALTER TABLE comments
ALTER COLUMN user_id DROP NOT NULL;

ALTER TABLE comments
ADD COLUMN IF NOT EXISTS name varchar(100) NOT NULL default '';