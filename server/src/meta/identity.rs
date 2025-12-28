// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use chrono::{DateTime, Utc};
use geist_sdk::pb::meta::v1alpha::{
    identity_provider::IdentityProvider as ProtoIdentityProvider,
    identity_service_server::IdentityService, Identity, IdentityProvider, IdentityRequest,
    IdentityResponse, LinkIdentityRequest, ListIdentitiesRequest, SetPrimaryIdentityRequest,
    UnlinkIdentityRequest,
};
use prost_types::Timestamp;
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Row};
use tonic::{Request, Status};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UserIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
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

impl UserIdentity {
    fn to_proto(&self) -> Result<Identity, Status> {
        let provider = match self.provider.as_str() {
            "google" => ProtoIdentityProvider::Google as i32,
            "github" => ProtoIdentityProvider::Github as i32,
            "twitter" => ProtoIdentityProvider::Twitter as i32,
            "discord" => ProtoIdentityProvider::Discord as i32,
            "apple" => ProtoIdentityProvider::Apple as i32,
            "microsoft" => ProtoIdentityProvider::Microsoft as i32,
            "email" => ProtoIdentityProvider::Email as i32,
            _ => return Err(Status::internal("Invalid provider type")),
        };

        Ok(Identity {
            uid: self.id.to_string(),
            user_uid: self.user_id.to_string(),
            provider: provider,
            provider_user_id: self.provider_user_id.clone(),
            provider_email: self.provider_email.clone().unwrap_or_default(),
            provider_username: self.provider_username.clone().unwrap_or_default(),
            provider_avatar_url: self.provider_avatar_url.clone().unwrap_or_default(),
            is_primary: self.is_primary,
            verified: self.verified,
            create_time: Some(Timestamp {
                seconds: self.create_time.timestamp(),
                nanos: self.create_time.timestamp_subsec_nanos() as i32,
            }),
            update_time: Some(Timestamp {
                seconds: self.update_time.timestamp(),
                nanos: self.update_time.timestamp_subsec_nanos() as i32,
            }),
            last_used_at: self.last_used_at.map(|dt| Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }),
        })
    }

    fn provider_to_string(provider: i32) -> Result<String, Status> {
        match provider {
            x if x == ProtoIdentityProvider::Google as i32 => Ok("google".to_string()),
            x if x == ProtoIdentityProvider::Github as i32 => Ok("github".to_string()),
            x if x == ProtoIdentityProvider::Twitter as i32 => Ok("twitter".to_string()),
            x if x == ProtoIdentityProvider::Discord as i32 => Ok("discord".to_string()),
            x if x == ProtoIdentityProvider::Apple as i32 => Ok("apple".to_string()),
            x if x == ProtoIdentityProvider::Microsoft as i32 => Ok("microsoft".to_string()),
            x if x == ProtoIdentityProvider::Email as i32 => Ok("email".to_string()),
            _ => Err(Status::invalid_argument("Invalid identity provider")),
        }
    }
}

pub struct IdentityRepository {
    pool: PgPool,
}

impl IdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, 
                   provider_username, provider_avatar_url, access_token_encrypted,
                   refresh_token_encrypted, token_expires_at, metadata, is_primary,
                   verified, create_time, update_time, last_used_at
            FROM public.user_identities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_provider(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, 
                   provider_username, provider_avatar_url, access_token_encrypted,
                   refresh_token_encrypted, token_expires_at, metadata, is_primary,
                   verified, create_time, update_time, last_used_at
            FROM public.user_identities
            WHERE provider = $1 AND provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserIdentity>, sqlx::Error> {
        sqlx::query_as::<_, UserIdentity>(
            r#"
            SELECT id, user_id, provider, provider_user_id, provider_email, 
                   provider_username, provider_avatar_url, access_token_encrypted,
                   refresh_token_encrypted, token_expires_at, metadata, is_primary,
                   verified, create_time, update_time, last_used_at
            FROM public.user_identities
            WHERE user_id = $1
            ORDER BY is_primary DESC, create_time ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, identity: &CreateIdentity) -> Result<UserIdentity, sqlx::Error> {
        let now = Utc::now();
        let id = Uuid::now_v7();

        sqlx::query_as::<_, UserIdentity>(
            r#"
            INSERT INTO public.user_identities 
                (id, user_id, provider, provider_user_id, provider_email, 
                 provider_username, provider_avatar_url, access_token_encrypted,
                 refresh_token_encrypted, token_expires_at, metadata, is_primary,
                 verified, create_time, update_time)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, user_id, provider, provider_user_id, provider_email, 
                      provider_username, provider_avatar_url, access_token_encrypted,
                      refresh_token_encrypted, token_expires_at, metadata, is_primary,
                      verified, create_time, update_time, last_used_at
            "#,
        )
        .bind(id)
        .bind(identity.user_id)
        .bind(&identity.provider)
        .bind(&identity.provider_user_id)
        .bind(&identity.provider_email)
        .bind(&identity.provider_username)
        .bind(&identity.provider_avatar_url)
        .bind(&identity.access_token_encrypted)
        .bind(&identity.refresh_token_encrypted)
        .bind(identity.token_expires_at)
        .bind(&identity.metadata)
        .bind(identity.is_primary)
        .bind(identity.verified)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM public.user_identities
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_primary(&self, id: Uuid, user_id: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Unset all primary flags for this user
        sqlx::query(
            r#"
            UPDATE public.user_identities
            SET is_primary = false, update_time = now()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Set this identity as primary
        sqlx::query(
            r#"
            UPDATE public.user_identities
            SET is_primary = true, update_time = now()
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        // Update user's primary_identity_id
        sqlx::query(
            r#"
            UPDATE public.users
            SET primary_identity_id = $1, update_time = now()
            WHERE id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE public.user_identities
            SET last_used_at = now(), update_time = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn count_by_user_id(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM public.user_identities
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}

pub struct CreateIdentity {
    pub user_id: Uuid,
    pub provider: String,
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
}

#[derive(Debug, Default)]
pub struct IdentityServer {
    pool: Option<PgPool>,
}

impl IdentityServer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool: Some(pool) }
    }

    fn pool(&self) -> Result<&PgPool, Status> {
        self.pool
            .as_ref()
            .ok_or_else(|| Status::internal("Database pool not initialized"))
    }
}

#[tonic::async_trait]
impl IdentityService for IdentityServer {
    #[tracing::instrument(skip(self))]
    async fn get_identity(
        &self,
        request: Request<IdentityRequest>,
    ) -> ServerResult<IdentityResponse> {
        let req = request.into_inner();
        let repo = IdentityRepository::new(self.pool()?.clone());

        // Handle oneof params - prost generates oneof as Option<enum>
        // We'll use a helper to extract the value
        let identity = match req.params {
            Some(geist_sdk::pb::meta::v1alpha::identity_request::Params::Uid(uid)) => {
                let id = Uuid::parse_str(&uid)
                    .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;
                repo.find_by_id(id)
                    .await
                    .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            }
            Some(geist_sdk::pb::meta::v1alpha::identity_request::Params::UserUid(user_uid)) => {
                let user_id = Uuid::parse_str(&user_uid)
                    .map_err(|e| Status::invalid_argument(format!("Invalid UUID: {}", e)))?;
                let identities = repo
                    .find_by_user_id(user_id)
                    .await
                    .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
                if identities.is_empty() {
                    None
                } else {
                    Some(identities[0].clone())
                }
            }
            Some(geist_sdk::pb::meta::v1alpha::identity_request::Params::ProviderUserId(
                provider_user_id,
            )) => {
                let provider = UserIdentity::provider_to_string(req.provider)?;
                repo.find_by_provider(&provider, &provider_user_id)
                    .await
                    .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            }
            None => {
                return Err(Status::invalid_argument(
                    "One of uid, user_uid, or provider_user_id must be provided",
                ));
            }
        };

        let identities = if let Some(ident) = identity {
            vec![ident.to_proto()?]
        } else {
            vec![]
        };

        Ok(tonic::Response::new(IdentityResponse {
            identities,
            page: None,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn list_identities(
        &self,
        request: Request<ListIdentitiesRequest>,
    ) -> ServerResult<IdentityResponse> {
        let req = request.into_inner();
        let repo = IdentityRepository::new(self.pool()?.clone());

        let user_id = Uuid::parse_str(&req.user_uid)
            .map_err(|e| Status::invalid_argument(format!("Invalid user UUID: {}", e)))?;

        let identities = repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let proto_identities: Result<Vec<_>, _> = identities.iter().map(|i| i.to_proto()).collect();
        let proto_identities = proto_identities?;

        Ok(tonic::Response::new(IdentityResponse {
            identities: proto_identities,
            page: None,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn link_identity(
        &self,
        request: Request<LinkIdentityRequest>,
    ) -> ServerResult<IdentityResponse> {
        let req = request.into_inner();
        let repo = IdentityRepository::new(self.pool()?.clone());

        let provider = UserIdentity::provider_to_string(req.provider)?;

        // Check if identity already exists
        if let Some(existing) = repo
            .find_by_provider(&provider, &req.provider_user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
        {
            // Update last used
            repo.update_last_used(existing.id)
                .await
                .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

            let updated = repo
                .find_by_id(existing.id)
                .await
                .map_err(|e| Status::internal(format!("Database error: {}", e)))?
                .ok_or_else(|| Status::internal("Identity not found after update"))?;

            return Ok(tonic::Response::new(IdentityResponse {
                identities: vec![updated.to_proto()?],
                page: None,
            }));
        }

        // Determine user_id
        let user_id = if let Some(user_uid) = req.user_uid {
            Uuid::parse_str(&user_uid)
                .map_err(|e| Status::invalid_argument(format!("Invalid user UUID: {}", e)))?
        } else {
            // Check if user exists by email (for account merging)
            // For now, create a new user - this can be enhanced later
            // TODO: Implement user creation or lookup logic
            return Err(Status::unimplemented(
                "Automatic user creation not yet implemented. Provide user_uid.",
            ));
        };

        // Check if user already has this provider linked
        let existing_identities = repo
            .find_by_user_id(user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let is_primary = existing_identities.is_empty();

        let create_identity = CreateIdentity {
            user_id,
            provider,
            provider_user_id: req.provider_user_id,
            provider_email: if req.provider_email.is_empty() {
                None
            } else {
                Some(req.provider_email)
            },
            provider_username: if req.provider_username.is_empty() {
                None
            } else {
                Some(req.provider_username)
            },
            provider_avatar_url: if req.provider_avatar_url.is_empty() {
                None
            } else {
                Some(req.provider_avatar_url)
            },
            access_token_encrypted: if req.access_token.is_empty() {
                None
            } else {
                // TODO: Encrypt token before storing
                Some(req.access_token)
            },
            refresh_token_encrypted: if req.refresh_token.is_empty() {
                None
            } else {
                // TODO: Encrypt token before storing
                Some(req.refresh_token)
            },
            token_expires_at: req.token_expires_at.map(|ts| {
                DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                    .unwrap_or_else(Utc::now)
            }),
            metadata: None,
            is_primary,
            verified: req.verified,
        };

        let identity = repo
            .create(&create_identity)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        // If this is the primary identity, update user's primary_identity_id
        if is_primary {
            sqlx::query(
                r#"
                UPDATE public.users
                SET primary_identity_id = $1, update_time = now()
                WHERE id = $2
                "#,
            )
            .bind(identity.id)
            .bind(user_id)
            .execute(self.pool()?)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        }

        Ok(tonic::Response::new(IdentityResponse {
            identities: vec![identity.to_proto()?],
            page: None,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn unlink_identity(
        &self,
        request: Request<UnlinkIdentityRequest>,
    ) -> ServerResult<IdentityResponse> {
        let req = request.into_inner();
        let repo = IdentityRepository::new(self.pool()?.clone());

        let identity_id = Uuid::parse_str(&req.identity_uid)
            .map_err(|e| Status::invalid_argument(format!("Invalid identity UUID: {}", e)))?;
        let user_id = Uuid::parse_str(&req.user_uid)
            .map_err(|e| Status::invalid_argument(format!("Invalid user UUID: {}", e)))?;

        // Check if this is the only identity
        let count = repo
            .count_by_user_id(user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        if count <= 1 {
            return Err(Status::failed_precondition(
                "Cannot unlink the last identity for a user",
            ));
        }

        // Verify the identity belongs to the user
        let identity = repo
            .find_by_id(identity_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Identity not found"))?;

        if identity.user_id != user_id {
            return Err(Status::permission_denied(
                "Identity does not belong to the specified user",
            ));
        }

        // If this is the primary identity, set another one as primary
        if identity.is_primary {
            let identities = repo
                .find_by_user_id(user_id)
                .await
                .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

            if let Some(new_primary) = identities.iter().find(|i| i.id != identity_id) {
                repo.set_primary(new_primary.id, user_id)
                    .await
                    .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
            }
        }

        // Delete the identity
        repo.delete(identity_id, user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        Ok(tonic::Response::new(IdentityResponse {
            identities: vec![],
            page: None,
        }))
    }

    #[tracing::instrument(skip(self))]
    async fn set_primary_identity(
        &self,
        request: Request<SetPrimaryIdentityRequest>,
    ) -> ServerResult<IdentityResponse> {
        let req = request.into_inner();
        let repo = IdentityRepository::new(self.pool()?.clone());

        let identity_id = Uuid::parse_str(&req.identity_uid)
            .map_err(|e| Status::invalid_argument(format!("Invalid identity UUID: {}", e)))?;
        let user_id = Uuid::parse_str(&req.user_uid)
            .map_err(|e| Status::invalid_argument(format!("Invalid user UUID: {}", e)))?;

        // Verify the identity belongs to the user
        let identity = repo
            .find_by_id(identity_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Identity not found"))?;

        if identity.user_id != user_id {
            return Err(Status::permission_denied(
                "Identity does not belong to the specified user",
            ));
        }

        repo.set_primary(identity_id, user_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let updated = repo
            .find_by_id(identity_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::internal("Identity not found after update"))?;

        Ok(tonic::Response::new(IdentityResponse {
            identities: vec![updated.to_proto()?],
            page: None,
        }))
    }
}

