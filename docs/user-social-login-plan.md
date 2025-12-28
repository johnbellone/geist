# Implementation Plan: Multiple Social Logins to Single User Profile

## Overview

This plan outlines the implementation for supporting multiple social login providers (Google, GitHub, Twitter, etc.) that all map to a single user profile. This enables users to authenticate with any of their linked accounts while maintaining a unified identity.

## Architecture Principles

1. **User Profile as Primary Identity**: The `users` table represents the canonical user profile
2. **Identity Providers as Secondary**: Multiple identity providers link to a single user
3. **Provider-Agnostic Authentication**: Users can authenticate via any linked provider
4. **Account Linking**: Users can link/unlink additional providers after initial signup
5. **Email Uniqueness**: Primary email remains unique, but provider emails can overlap

## 1. Database Schema Changes

### 1.1 New Table: `user_identities`

Stores provider-specific identity information linked to users.

```sql
-- Migration: user_identities.up.sql

-- Provider types enum
create type if not exists public.identity_provider as enum (
    'google',
    'github',
    'twitter',
    'discord',
    'apple',
    'microsoft',
    'email'  -- For email/password auth
);

-- User identities table
create table if not exists public.user_identities (
    id uuid not null default gen_random_uuid() primary key,
    user_id uuid not null references public.users(id) on delete cascade,
    provider identity_provider not null,
    provider_user_id text not null,  -- Provider's unique identifier for this user
    provider_email citext,           -- Email from provider (may differ from user.email)
    provider_username text,          -- Username from provider
    provider_avatar_url text,        -- Avatar URL from provider
    access_token_encrypted text,     -- Encrypted OAuth access token (optional)
    refresh_token_encrypted text,    -- Encrypted OAuth refresh token (optional)
    token_expires_at timestamp,      -- When access token expires
    metadata jsonb,                  -- Provider-specific metadata (JSON)
    is_primary boolean not null default false,  -- Primary identity for this user
    verified boolean not null default false,   -- Whether provider verified this identity
    create_time timestamp not null default now(),
    update_time timestamp not null default now(),
    last_used_at timestamp,          -- Last time this identity was used for auth
    
    -- Unique constraint: one identity per provider per user
    unique(user_id, provider),
    
    -- Unique constraint: provider_user_id must be unique per provider
    unique(provider, provider_user_id)
);

-- Indexes
create index if not exists idx_user_identities_user_id on public.user_identities(user_id);
create index if not exists idx_user_identities_provider on public.user_identities(provider);
create index if not exists idx_user_identities_provider_user_id on public.user_identities(provider, provider_user_id);
create index if not exists idx_user_identities_provider_email on public.user_identities(provider_email) where provider_email is not null;
create index if not exists idx_user_identities_primary on public.user_identities(user_id, is_primary) where is_primary = true;

-- Ensure only one primary identity per user
create unique index if not exists idx_user_identities_one_primary 
    on public.user_identities(user_id) 
    where is_primary = true;
```

### 1.2 Update `users` Table

Modify the users table to support the new identity model:

```sql
-- Migration: users_update_for_identities.up.sql

-- Add uid column (if not exists) for UUID-based lookups
alter table public.users 
    add column if not exists uid uuid not null default gen_random_uuid() unique;

-- Make name nullable (users might not have a name initially)
alter table public.users 
    alter column name drop not null;

-- Make email nullable (email might come from identity provider)
alter table public.users 
    alter column email drop not null;

-- Add primary_email column to track the canonical email
alter table public.users 
    add column if not exists primary_email citext;

-- Add primary_identity_id to track the primary authentication method
alter table public.users 
    add column if not exists primary_identity_id uuid references public.user_identities(id);

-- Update indexes
create index if not exists idx_users_uid on public.users(uid);
create index if not exists idx_users_primary_email on public.users(primary_email) where primary_email is not null;
```

### 1.3 Rollback Migrations

```sql
-- user_identities.down.sql
drop index if exists idx_user_identities_one_primary;
drop index if exists idx_user_identities_primary;
drop index if exists idx_user_identities_provider_email;
drop index if exists idx_user_identities_provider_user_id;
drop index if exists idx_user_identities_provider;
drop index if exists idx_user_identities_user_id;
drop table if exists public.user_identities;
drop type if exists public.identity_provider;
```

## 2. Protocol Buffer Changes

### 2.1 New Proto: `identity.proto`

```protobuf
// SPDX-License-Identifier: Apache-2.0
syntax = "proto3";
package geist.meta.v1alpha;

import "buf/validate/validate.proto";
import "google/protobuf/timestamp.proto";
import "geist/rpc/pagination.proto";

// Identity provider types
enum IdentityProvider {
    IDENTITY_PROVIDER_UNSPECIFIED = 0;
    IDENTITY_PROVIDER_GOOGLE = 1;
    IDENTITY_PROVIDER_GITHUB = 2;
    IDENTITY_PROVIDER_TWITTER = 3;
    IDENTITY_PROVIDER_DISCORD = 4;
    IDENTITY_PROVIDER_APPLE = 5;
    IDENTITY_PROVIDER_MICROSOFT = 6;
    IDENTITY_PROVIDER_EMAIL = 7;  // Email/password authentication
}

// Identity represents a linked authentication provider
message Identity {
    string uid = 1 [(buf.validate.field).string.uuid = true];
    string user_uid = 2 [(buf.validate.field).string.uuid = true];
    IdentityProvider provider = 3;
    string provider_user_id = 4;  // Provider's unique ID
    string provider_email = 5;
    string provider_username = 6;
    string provider_avatar_url = 7;
    bool is_primary = 8;
    bool verified = 9;
    google.protobuf.Timestamp create_time = 10;
    google.protobuf.Timestamp update_time = 11;
    google.protobuf.Timestamp last_used_at = 12;
    // Note: Access tokens are never returned in API responses
}

// IdentityService manages user identity providers
service IdentityService {
    rpc GetIdentity(IdentityRequest) returns (IdentityResponse);
    rpc ListIdentities(ListIdentitiesRequest) returns (IdentityResponse);
    rpc LinkIdentity(LinkIdentityRequest) returns (IdentityResponse);
    rpc UnlinkIdentity(UnlinkIdentityRequest) returns (IdentityResponse);
    rpc SetPrimaryIdentity(SetPrimaryIdentityRequest) returns (IdentityResponse);
}

message IdentityRequest {
    oneof params {
        string uid = 1 [(buf.validate.field).string.uuid = true];
        string user_uid = 2 [(buf.validate.field).string.uuid = true];
        string provider_user_id = 3;
    }
    IdentityProvider provider = 4;
}

message IdentityResponse {
    repeated Identity identities = 1;
    geist.rpc.Pagination page = 2;
}

message ListIdentitiesRequest {
    string user_uid = 1 [(buf.validate.field).string.uuid = true];
    geist.rpc.Pagination page = 2;
}

message LinkIdentityRequest {
    IdentityProvider provider = 1;
    string provider_user_id = 2;
    string provider_email = 3;
    string provider_username = 4;
    string provider_avatar_url = 5;
    bool verified = 6;
    // OAuth tokens (encrypted before storage)
    string access_token = 7;
    string refresh_token = 8;
    google.protobuf.Timestamp token_expires_at = 9;
    // Optional: link to existing user by UID
    string user_uid = 10;
}

message UnlinkIdentityRequest {
    string identity_uid = 1 [(buf.validate.field).string.uuid = true];
    string user_uid = 2 [(buf.validate.field).string.uuid = true];
}

message SetPrimaryIdentityRequest {
    string identity_uid = 1 [(buf.validate.field).string.uuid = true];
    string user_uid = 2 [(buf.validate.field).string.uuid = true];
}
```

### 2.2 Update `user.proto`

Add linked identities to the User message:

```protobuf
// Add to existing User message
message User {
    // ... existing fields ...
    
    // Linked identities (populated on request)
    repeated Identity identities = 11;
    Identity primary_identity = 12;
}
```

## 3. Service Implementation

### 3.1 Database Models

Create Rust structs for database operations:

```rust
// server/src/meta/identity.rs

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,  // Will be converted to enum
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub provider_username: Option<String>,
    pub provider_avatar_url: Option<String>,
    pub access_token_encrypted: Option<String>,
    pub refresh_token_encrypted: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<Value>,
    pub is_primary: bool,
    pub verified: bool,
    pub create_time: DateTime<Utc>,
    pub update_time: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
```

### 3.2 Identity Service Implementation

Key operations:

1. **Link Identity**: 
   - Check if identity already exists (by provider + provider_user_id)
   - If exists, link to existing user or merge users
   - If new, create identity and optionally create user
   - Handle email conflicts

2. **Unlink Identity**:
   - Verify user owns the identity
   - Prevent unlinking if it's the only identity
   - Prevent unlinking primary identity if multiple exist

3. **Set Primary Identity**:
   - Verify user owns the identity
   - Update primary flag

4. **Find User by Identity**:
   - Lookup user by provider + provider_user_id
   - Used during authentication flow

### 3.3 Authentication Flow

```
1. User authenticates with Provider (OAuth/OIDC)
2. Provider returns user info + tokens
3. Server looks up identity by (provider, provider_user_id)
4. If identity exists:
   - Update last_used_at
   - Return linked user
5. If identity doesn't exist:
   - Check if user exists by email (optional merge)
   - Create new user + identity
   - Return new user
6. Generate JWT token for user
7. Return token to client
```

## 4. Implementation Phases

### Phase 1: Database Schema
- [ ] Create `user_identities` table migration
- [ ] Update `users` table migration
- [ ] Create rollback migrations
- [ ] Test migrations up/down

### Phase 2: Protocol Buffers
- [ ] Create `identity.proto`
- [ ] Update `user.proto` with identity fields
- [ ] Regenerate Rust code from protos
- [ ] Update SDK build configuration

### Phase 3: Database Layer
- [ ] Create `UserIdentity` struct
- [ ] Implement identity repository/traits
- [ ] Implement CRUD operations
- [ ] Add encryption for tokens (if needed)

### Phase 4: Identity Service
- [ ] Implement `IdentityService` gRPC service
- [ ] Implement `LinkIdentity` operation
- [ ] Implement `UnlinkIdentity` operation
- [ ] Implement `SetPrimaryIdentity` operation
- [ ] Implement `ListIdentities` operation
- [ ] Implement `GetIdentity` operation

### Phase 5: User Service Updates
- [ ] Update `GetUser` to include identities (optional)
- [ ] Update `CreateUser` to handle identity linking
- [ ] Add identity lookup helpers

### Phase 6: Authentication Integration
- [ ] Update `TokenInterceptor` to validate JWT and extract user
- [ ] Create identity lookup helper for auth
- [ ] Implement OAuth callback handlers (future: HTTP endpoints)
- [ ] Add user context extraction middleware

### Phase 7: Testing
- [ ] Unit tests for identity operations
- [ ] Integration tests for linking/unlinking
- [ ] Test user merge scenarios
- [ ] Test authentication flows

## 5. Security Considerations

1. **Token Encryption**: OAuth tokens stored encrypted at rest
2. **Token Rotation**: Handle token refresh flows
3. **Email Verification**: Track provider-verified emails
4. **Account Merging**: Require explicit confirmation for merging accounts
5. **Primary Identity**: Prevent account lockout (require at least one identity)
6. **Rate Limiting**: Prevent identity enumeration attacks

## 6. Configuration

Add to `AppConfig`:

```rust
/// OAuth encryption key (for encrypting stored tokens)
#[arg(long, env = "OAUTH_ENCRYPTION_KEY")]
pub oauth_encryption_key: Option<String>,

/// Database connection string
#[arg(long, env = "DATABASE_URL")]
pub database_url: String,
```

## 7. Future Enhancements

1. **OAuth Provider Configuration**: Dynamic provider setup
2. **Token Refresh**: Automatic token refresh before expiry
3. **Account Merging UI**: User-facing account linking interface
4. **Identity Verification**: Additional verification steps
5. **Audit Logging**: Track identity linking/unlinking events

## 8. Migration Strategy

1. **Backward Compatibility**: Existing users without identities continue to work
2. **Data Migration**: Create default email identity for existing users
3. **Gradual Rollout**: Support both old and new auth methods during transition

