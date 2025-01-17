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

use cadence::{Counted, Gauged, StatsdClient};

use super::ServerMetricExt;
use crate::listen::{ListenSnapshot, ListenStats};

const METRIC_NAME_LISTEN_INSTANCE_COUNT: &str = "listen.instance.count";
const METRIC_NAME_LISTEN_ACCEPTED: &str = "listen.accepted";
const METRIC_NAME_LISTEN_DROPPED: &str = "listen.dropped";
const METRIC_NAME_LISTEN_TIMEOUT: &str = "listen.timeout";
const METRIC_NAME_LISTEN_FAILED: &str = "listen.failed";

pub fn emit_listen_stats(
    client: &StatsdClient,
    stats: &Arc<ListenStats>,
    snap: &mut ListenSnapshot,
) {
    let online_value = if stats.is_running() { "y" } else { "n" };
    let server = stats.name();
    let mut buffer = itoa::Buffer::new();
    let stat_id = buffer.format(stats.stat_id().as_u64());

    client
        .gauge_with_tags(
            METRIC_NAME_LISTEN_INSTANCE_COUNT,
            stats.get_running_runtime_count() as f64,
        )
        .add_server_tags(server, online_value, stat_id)
        .send();

    let new_value = stats.get_accepted();
    if new_value != 0 || snap.accepted != 0 {
        let diff_value = i64::try_from(new_value.wrapping_sub(snap.accepted)).unwrap_or(i64::MAX);
        client
            .count_with_tags(METRIC_NAME_LISTEN_ACCEPTED, diff_value)
            .add_server_tags(server, online_value, stat_id)
            .send();
        snap.accepted = new_value;
    }

    let new_value = stats.get_dropped();
    if new_value != 0 || snap.dropped != 0 {
        let diff_value = i64::try_from(new_value.wrapping_sub(snap.dropped)).unwrap_or(i64::MAX);
        client
            .count_with_tags(METRIC_NAME_LISTEN_DROPPED, diff_value)
            .add_server_tags(server, online_value, stat_id)
            .send();
        snap.dropped = new_value;
    }

    let new_value = stats.get_timeout();
    if new_value != 0 || snap.timeout != 0 {
        let diff_value = i64::try_from(new_value.wrapping_sub(snap.timeout)).unwrap_or(i64::MAX);
        client
            .count_with_tags(METRIC_NAME_LISTEN_TIMEOUT, diff_value)
            .add_server_tags(server, online_value, stat_id)
            .send();
        snap.timeout = new_value;
    }

    let new_value = stats.get_failed();
    if new_value != 0 || snap.failed != 0 {
        let diff_value = i64::try_from(new_value.wrapping_sub(snap.failed)).unwrap_or(i64::MAX);
        client
            .count_with_tags(METRIC_NAME_LISTEN_FAILED, diff_value)
            .add_server_tags(server, online_value, stat_id)
            .send();
        snap.failed = new_value;
    }
}
