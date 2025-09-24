// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::pb::meta::v1alpha::{
    user_service_server::UserService, ListUsersRequest, MutateUserRequest, UserRequest,
    UserResponse,
};
use tonic::{Request, Status};

#[derive(Debug, Default)]
pub struct UserServer;

#[tonic::async_trait]
impl UserService for UserServer {
    #[tracing::instrument]
    async fn get_user(&self, _req: Request<UserRequest>) -> ServerResult<UserResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn list_users(&self, _req: Request<ListUsersRequest>) -> ServerResult<UserResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn create_user(&self, _req: Request<MutateUserRequest>) -> ServerResult<UserResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn update_user(&self, _req: Request<MutateUserRequest>) -> ServerResult<UserResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn delete_user(&self, _req: Request<MutateUserRequest>) -> ServerResult<UserResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }
}
