// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod meta;

use tonic::{metadata::AsciiMetadataValue, Request};
use tracing::debug;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{filter::Targets, registry::LookupSpan, Layer};
use uuid::Uuid;

pub type ServerResult<T> = Result<tonic::Response<T>, tonic::Status>;
pub type InterceptResult<T> = Result<tonic::Request<T>, tonic::Status>;

#[derive(Clone, Debug, Default)]
pub struct TraceInterceptor;

impl tonic::service::Interceptor for TraceInterceptor {
    #[tracing::instrument]
    fn call(&mut self, mut req: Request<()>) -> InterceptResult<()> {
        if !req.metadata().contains_key("x-trace-id") {
            let uuid: String = Uuid::now_v7().to_string();
            let value = AsciiMetadataValue::try_from(uuid).unwrap();
            req.metadata_mut().insert("x-trace-id", value.clone());
            debug!("Trace-Id: {:?}", value);
        }

        Ok(req)
    }
}

#[derive(Clone, Debug, Default)]
pub struct TokenInterceptor;

impl tonic::service::Interceptor for TokenInterceptor {
    #[tracing::instrument]
    fn call(&mut self, req: Request<()>) -> InterceptResult<()> {
        match req.metadata().get("authorization") {
            Some(token) => {
                debug!("Authorization token: {:?}", token);
                Ok(req)
            }
            _ => Err(tonic::Status::unauthenticated("invalid token")),
        }
    }
}

struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = event.metadata().level();
        let target = event.metadata().target();
        let name = event.metadata().name();

        metrics::counter!(
            "tracing_events_total",
            "level" => level.to_string(),
            "target" => target.to_string(),
            "name" => name.to_string()
        )
        .increment(1);

        if *level == Level::ERROR {
            metrics::counter!(
                "tracing_errors_total",
                "target" => target.to_string(),
                "name" => name.to_string()
            )
            .increment(1);
        }
    }
}

pub fn tracing_metrics_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    MetricsLayer.with_filter(Targets::new().with_target("geist-server", Level::INFO))
}
