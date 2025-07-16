// SPDX-License-Identifier: Apache-2.0

pub mod geist {
    pub mod meta {
        pub mod v1alpha {
            tonic::include_proto!("geist.meta.v1alpha");
        }
    }

    pub mod rpc {
        tonic::include_proto!("geist.rpc");
    }
}
