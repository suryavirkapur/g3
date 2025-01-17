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

use std::sync::Arc;

use g3_daemon::stat::task::UdpConnectConnectionStats;

use crate::module::udp_connect::{ArcUdpConnectTaskRemoteStats, UdpConnectTaskRemoteStats};

#[derive(Default)]
pub(crate) struct UdpConnectTaskStats {
    pub(crate) clt: UdpConnectConnectionStats,
    pub(crate) ups: UdpConnectConnectionStats,
}

impl UdpConnectTaskStats {
    #[inline]
    pub(crate) fn for_escaper(self: &Arc<Self>) -> ArcUdpConnectTaskRemoteStats {
        Arc::clone(self) as ArcUdpConnectTaskRemoteStats
    }
}

impl UdpConnectTaskRemoteStats for UdpConnectTaskStats {
    fn add_recv_bytes(&self, size: u64) {
        self.ups.recv.add_bytes(size);
    }

    fn add_recv_packet(&self) {
        self.ups.recv.add_packet();
    }

    fn add_send_bytes(&self, size: u64) {
        self.ups.send.add_bytes(size);
    }

    fn add_send_packet(&self) {
        self.ups.send.add_packet();
    }
}
