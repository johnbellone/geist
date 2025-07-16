// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::geist::meta::v1alpha::{
    user_service_server::UserService, UserRequest, UserResponse,
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
}
