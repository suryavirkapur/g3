/*
 * Copyright 2023 ByteDance and/or its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use slog::Logger;

use g3_daemon::server::ClientConnectionInfo;

use crate::config::server::openssl_proxy::OpensslProxyServerConfig;
use crate::serve::openssl_proxy::OpensslProxyServerStats;
use crate::serve::ServerQuitPolicy;

pub(crate) struct CommonTaskContext {
    pub server_config: Arc<OpensslProxyServerConfig>,
    pub server_stats: Arc<OpensslProxyServerStats>,
    pub server_quit_policy: Arc<ServerQuitPolicy>,
    pub cc_info: ClientConnectionInfo,
    pub task_logger: Logger,
}

impl CommonTaskContext {
    #[inline]
    pub(super) fn client_addr(&self) -> SocketAddr {
        self.cc_info.client_addr()
    }

    #[inline]
    pub(super) fn client_ip(&self) -> IpAddr {
        self.cc_info.client_ip()
    }
}
