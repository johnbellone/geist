// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::pb::meta::v1alpha::{
    feed_service_server::FeedService, FeedRequest, FeedResponse, ListFeedsRequest,
    MutateFeedRequest,
};
use tonic::{Request, Status};

#[derive(Debug, Default)]
pub struct FeedServer;

#[tonic::async_trait]
impl FeedService for FeedServer {
    #[tracing::instrument]
    async fn get_feed(&self, _req: Request<FeedRequest>) -> ServerResult<FeedResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn list_feeds(&self, _req: Request<ListFeedsRequest>) -> ServerResult<FeedResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn create_feed(&self, _req: Request<MutateFeedRequest>) -> ServerResult<FeedResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn update_feed(&self, _req: Request<MutateFeedRequest>) -> ServerResult<FeedResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }

    #[tracing::instrument]
    async fn delete_feed(&self, _req: Request<MutateFeedRequest>) -> ServerResult<FeedResponse> {
        Err(Status::unimplemented("rpc not implemented"))
    }
}
