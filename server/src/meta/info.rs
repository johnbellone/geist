// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::pb::{
    info_service_server::InfoService, InfoRequest, InfoResponse,
};
use tonic::{Request, Status};

#[derive(Debug, Default)]
pub struct InfoServer;

#[tonic::async_trait]
impl InfoService for InfoServer {
    #[tracing::instrument]
    async fn get_info(&self, _req: Request<InfoRequest>) -> ServerResult<InfoResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }
}