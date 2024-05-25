// SPDX-License-Identifier: Apache-2.0

pub mod iam;
pub mod meta;

pub type ServerResult<T> = Result<tonic::Response<T>, tonic::Status>;