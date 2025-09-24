create type if not exists public.group_visibility as enum ('internal', 'public', 'private', 'preview', 'global');

create table if not exists public.groups (
    id uuid not null default gen_random_uuid(),
    name citext not null primary key,
    description text,
    slug citext not null unique,
    icon_url text,
    visibility group_visibility not null,
    create_time timestamp not null default now(),
    update_time timestamp not null default now()
);

create index if not exists idx_groups_id on public.groups(id);
create index if not exists idx_groups_slug on public.groups(slug);