// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::geist::meta::v1alpha::{
    role_service_server::RoleService, RoleRequest, RoleResponse,
};
use tonic::{Request, Status};

#[derive(Debug, Default)]
pub struct RoleServer;

#[tonic::async_trait]
impl RoleService for RoleServer {
    #[tracing::instrument]
    async fn get_role(&self, _req: Request<RoleRequest>) -> ServerResult<RoleResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }
}
