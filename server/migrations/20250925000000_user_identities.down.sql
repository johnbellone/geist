-- Drop indexes
drop index if exists idx_user_identities_one_primary;
drop index if exists idx_user_identities_primary;
drop index if exists idx_user_identities_provider_email;
drop index if exists idx_user_identities_provider_user_id;
drop index if exists idx_user_identities_provider;
drop index if exists idx_user_identities_user_id;

-- Drop foreign key and column
alter table public.users 
    drop column if exists primary_identity_id;

-- Drop table
drop table if exists public.user_identities;

-- Revert users table changes
drop index if exists idx_users_primary_email;
drop index if exists idx_users_uid;
alter table public.users 
    drop column if exists primary_email;
alter table public.users 
    alter column email set not null;
alter table public.users 
    alter column name set not null;
alter table public.users 
    drop column if exists uid;

-- Drop enum
drop type if exists public.identity_provider;

