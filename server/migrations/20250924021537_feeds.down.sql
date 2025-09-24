drop index if exists idx_feeds_id;
drop index if exists idx_feeds_url;
drop table if exists public.feeds;
drop type if exists public.feed_type;
drop type if exists public.feed_visibility;