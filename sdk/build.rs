// SPDX-License-Identifier: Apache-2.0

fn main() -> Result<(), tonic_buf_build::error::TonicBufBuildError> {
    tonic_buf_build::compile_from_buf(tonic_build::configure(), None)?;
    Ok(())
}