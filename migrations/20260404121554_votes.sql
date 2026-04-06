-- Create votes table for ideas
CREATE TABLE IF NOT EXISTS idea_votes (
    id bigserial primary key,
    idea_id bigint not null references ideas(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    vote_type smallint not null, -- 1 for upvote, -1 for downvote
    created_at timestamptz not null default now(),
    unique (idea_id, user_id)
);

-- Create index for faster vote queries
CREATE INDEX IF NOT EXISTS idx_idea_votes_idea_id ON idea_votes(idea_id);
