// SPDX-License-Identifier: Apache-2.0

use crate::ServerResult;
use geist_sdk::geist::meta::v1alpha::{
    feed_service_server::FeedService, FeedRequest, FeedResponse, ListFeedsRequest,
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
}
