// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::pb::meta::v1alpha::{
    group_service_server::GroupService, GroupRequest, GroupResponse, ListGroupsRequest,
    MutateGroupRequest,
};
use tonic::{Request, Status};

#[derive(Debug, Default)]
pub struct GroupServer;

#[tonic::async_trait]
impl GroupService for GroupServer {
    #[tracing::instrument]
    async fn get_group(&self, _req: Request<GroupRequest>) -> ServerResult<GroupResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn list_groups(&self, _req: Request<ListGroupsRequest>) -> ServerResult<GroupResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn create_group(&self, _req: Request<MutateGroupRequest>) -> ServerResult<GroupResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn update_group(&self, _req: Request<MutateGroupRequest>) -> ServerResult<GroupResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn delete_group(&self, _req: Request<MutateGroupRequest>) -> ServerResult<GroupResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }
}
