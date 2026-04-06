-- Create collaboration_requests table
CREATE TABLE IF NOT EXISTS collaboration_requests (
    id bigserial primary key,
    project_id bigint not null references projects(id) on delete cascade,
    requester_id bigint not null references users(id) on delete cascade,
    message text not null,
    requested_role varchar(50) not null,
    status varchar(20) not null default 'pending',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
