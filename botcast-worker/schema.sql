-- Create custom enum types
CREATE TYPE task_status AS ENUM ('PENDING', 'RUNNING', 'COMPLETED', 'FAILED');

-- Create extension for UUID support if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create users table (no dependencies)
CREATE TABLE users (
    id UUID PRIMARY KEY,
    auth_id TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    name TEXT
);

-- Create podcasts table (depends on users)
CREATE TABLE podcasts (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    user_id UUID REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    icon VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    description TEXT
);

-- Create episodes table (depends on podcasts and users)
CREATE TABLE episodes (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    audio_url TEXT,
    user_id UUID REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    podcast_id UUID NOT NULL REFERENCES podcasts(id) ON UPDATE CASCADE ON DELETE CASCADE,
    srt_url TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    sections JSONB NOT NULL,
    description TEXT,
    duration_sec INTEGER
);

-- Create corners table (depends on podcasts and users)
CREATE TABLE corners (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    requesting_mail BOOLEAN NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    mail_schema JSONB NOT NULL,
    podcast_id UUID NOT NULL REFERENCES podcasts(id) ON UPDATE CASCADE ON DELETE CASCADE
);

-- Create mails table (depends on corners and users)
CREATE TABLE mails (
    id UUID PRIMARY KEY,
    body JSONB NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    corner_id UUID NOT NULL REFERENCES corners(id) ON UPDATE CASCADE ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL
);

-- Create scripts table (depends on users)
CREATE TABLE scripts (
    id UUID PRIMARY KEY,
    template JSONB NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    arguments JSONB NOT NULL
);

-- Create tasks table (depends on users)
CREATE TABLE tasks (
    id UUID PRIMARY KEY,
    status task_status NOT NULL,
    args JSONB NOT NULL,
    user_id UUID REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE,
    execute_after TIMESTAMPTZ NOT NULL,
    executed_at TIMESTAMPTZ,
    executed_finished_at TIMESTAMPTZ,
    result JSONB,
    cron TEXT
);

-- Create indexes for better performance on foreign key columns
CREATE INDEX idx_podcasts_user_id ON podcasts(user_id);
CREATE INDEX idx_episodes_user_id ON episodes(user_id);
CREATE INDEX idx_episodes_podcast_id ON episodes(podcast_id);
CREATE INDEX idx_corners_user_id ON corners(user_id);
CREATE INDEX idx_corners_podcast_id ON corners(podcast_id);
CREATE INDEX idx_mails_user_id ON mails(user_id);
CREATE INDEX idx_mails_corner_id ON mails(corner_id);
CREATE INDEX idx_scripts_user_id ON scripts(user_id);
CREATE INDEX idx_tasks_user_id ON tasks(user_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_execute_after ON tasks(execute_after);