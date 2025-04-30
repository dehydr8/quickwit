use quickwit_cluster::{Cluster, ClusterMember};
use quickwit_config::NodeConfig;
use quickwit_config::service::QuickwitService;
use quickwit_cluster::{ChannelTransport, FailureDetectorConfig};
use std::collections::HashSet;

use quickwit_proto::indexing::CpuCapacity;

pub(super) async fn create_empty_cluster(
    config: &NodeConfig,
    services: &[QuickwitService],
) -> anyhow::Result<Cluster> {
    let self_node = ClusterMember {
        node_id: config.node_id.clone(),
        generation_id: quickwit_cluster::GenerationId::now(),
        is_ready: false,
        enabled_services: HashSet::from_iter(services.to_owned()),
        gossip_advertise_addr: config.gossip_advertise_addr,
        grpc_advertise_addr: config.grpc_advertise_addr,
        indexing_tasks: Vec::new(),
        indexing_cpu_capacity: CpuCapacity::zero(),
    };
    let cluster = Cluster::join(
        config.cluster_id.clone(),
        self_node,
        config.gossip_advertise_addr,
        Vec::new(),
        config.gossip_interval,
        FailureDetectorConfig::default(),
        &ChannelTransport::default(),
        None,
    )
    .await?;
    Ok(cluster)
}