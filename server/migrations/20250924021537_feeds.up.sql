create type if not exists public.feed_type as enum ('rss', 'atom', 'json', 'xml');
create type if not exists public.feed_visibility as enum ('internal', 'public', 'private', 'preview', 'global');

create table if not exists public.feeds (
    id uuid not null default gen_random_uuid(),
    name citext not null primary key,
    description text,
    url text not null,
    icon_url text,
    type feed_type not null,
    visibility feed_visibility not null,
    create_time timestamp not null default now(),
    update_time timestamp not null default now()
);

create index if not exists idx_feeds_id on public.feeds(id);
create index if not exists idx_feeds_url on public.feeds(url);