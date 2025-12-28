-- Create identity provider enum
create type if not exists public.identity_provider as enum (
    'google',
    'github',
    'twitter',
    'discord',
    'apple',
    'microsoft',
    'email'
);

-- Update users table to support identity model
alter table public.users 
    add column if not exists uid uuid not null default gen_random_uuid() unique;

-- Make name nullable (users might not have a name initially from provider)
alter table public.users 
    alter column name drop not null;

-- Make email nullable (email might come from identity provider)
alter table public.users 
    alter column email drop not null;

-- Add primary_email column to track the canonical email
alter table public.users 
    add column if not exists primary_email citext;

-- Add index for uid lookups
create index if not exists idx_users_uid on public.users(uid);
create index if not exists idx_users_primary_email on public.users(primary_email) where primary_email is not null;

-- User identities table
create table if not exists public.user_identities (
    id uuid not null default gen_random_uuid() primary key,
    user_id uuid not null references public.users(id) on delete cascade,
    provider identity_provider not null,
    provider_user_id text not null,
    provider_email citext,
    provider_username text,
    provider_avatar_url text,
    access_token_encrypted text,
    refresh_token_encrypted text,
    token_expires_at timestamp,
    metadata jsonb,
    is_primary boolean not null default false,
    verified boolean not null default false,
    create_time timestamp not null default now(),
    update_time timestamp not null default now(),
    last_used_at timestamp,
    
    -- Unique constraint: one identity per provider per user
    unique(user_id, provider),
    
    -- Unique constraint: provider_user_id must be unique per provider
    unique(provider, provider_user_id)
);

-- Add foreign key reference for primary_identity_id
alter table public.users 
    add column if not exists primary_identity_id uuid references public.user_identities(id);

-- Indexes for user_identities
create index if not exists idx_user_identities_user_id on public.user_identities(user_id);
create index if not exists idx_user_identities_provider on public.user_identities(provider);
create index if not exists idx_user_identities_provider_user_id on public.user_identities(provider, provider_user_id);
create index if not exists idx_user_identities_provider_email on public.user_identities(provider_email) where provider_email is not null;
create index if not exists idx_user_identities_primary on public.user_identities(user_id, is_primary) where is_primary = true;

-- Ensure only one primary identity per user
create unique index if not exists idx_user_identities_one_primary 
    on public.user_identities(user_id) 
    where is_primary = true;

