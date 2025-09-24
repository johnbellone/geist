create table if not exists public.users (
    id uuid not null default gen_random_uuid(),
    name citext not null primary key,
    email citext not null unique,
    username citext not null unique,
    avatar_url text,
    bio text,
    location text,
    links hstore,
    create_time timestamp not null default now(),
    update_time timestamp not null default now()
);

create index if not exists idx_users_id on public.users(id);
create index if not exists idx_users_email on public.users(email);
create index if not exists idx_users_username on public.users(username);