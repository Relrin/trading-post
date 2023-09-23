use std::{
    net::IpAddr,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use cdrs_tokio::cluster::connection_pool::ConnectionPoolConfig;
use cdrs_tokio::cluster::session::{
    NodeDistanceEvaluatorWrapper, ReconnectionPolicyWrapper, RetryPolicyWrapper,
    DEFAULT_TRANSPORT_BUFFER_SIZE,
};
use cdrs_tokio::cluster::{ConnectionManager, KeyspaceHolder};
use cdrs_tokio::compression::Compression;
use cdrs_tokio::frame::{Envelope, Version};
use cdrs_tokio::frame_encoding::ProtocolFrameEncodingFactory;
use cdrs_tokio::future::BoxFuture;
use cdrs_tokio::load_balancing::node_distance_evaluator::AllLocalNodeDistanceEvaluator;
use cdrs_tokio::retry::ConstantReconnectionPolicy;
use cdrs_tokio::{
    authenticators::{SaslAuthenticatorProvider, StaticPasswordAuthenticatorProvider},
    cluster::session::Session,
    cluster::{GenericClusterConfig, TcpConnectionManager},
    error::Result,
    load_balancing::RoundRobinLoadBalancingStrategy,
    retry::DefaultRetryPolicy,
    transport::TransportTcp,
    types::prelude::*,
};
use futures::FutureExt;
use tokio::sync::mpsc::Sender;

use crate::cli::CliOptions;

const EVENT_CHANNEL_CAPACITY: usize = 32;

pub type CassandraSession = Arc<
    Session<
        TransportTcp,
        VirtualConnectionManager,
        RoundRobinLoadBalancingStrategy<TransportTcp, VirtualConnectionManager>,
    >,
>;

pub async fn create_cassandra_session(opts: &CliOptions) -> CassandraSession {
    let authenticator = Arc::new(StaticPasswordAuthenticatorProvider::new(
        opts.cassandra_user.clone(),
        opts.cassandra_password.clone(),
    ));
    let mask = Ipv4Addr::new(255, 255, 255, 0);
    let actual = Ipv4Addr::new(127, 0, 0, 0);
    let cluster_config = VirtualClusterConfig {
        authenticator,
        mask,
        actual,
        version: Version::V5,
    };
    let nodes = opts
        .cassandra_nodes
        .split(',')
        .filter_map(|address| address.parse().ok())
        .collect::<Vec<SocketAddr>>();
    let reconnection_policy = Arc::new(ConstantReconnectionPolicy::default());

    let session = cdrs_tokio::cluster::connect_generic(
        &cluster_config,
        nodes,
        RoundRobinLoadBalancingStrategy::new(),
        RetryPolicyWrapper(Box::<DefaultRetryPolicy>::default()),
        ReconnectionPolicyWrapper(reconnection_policy),
        NodeDistanceEvaluatorWrapper(Box::<AllLocalNodeDistanceEvaluator>::default()),
        None,
    )
    .await
    .expect("session should be created");

    Arc::new(session)
}

// Implements a cluster configuration where the addresses to
// connect to are different from the ones configured by replacing
// the masked part of the address with a different subnet.
//
// This would allow running your connection through a proxy
// or mock server while also using a production configuration
// and having your load balancing configuration be aware of the
// 'real' addresses.
struct VirtualClusterConfig {
    authenticator: Arc<dyn SaslAuthenticatorProvider + Sync + Send>,
    mask: Ipv4Addr,
    actual: Ipv4Addr,
    version: Version,
}

fn rewrite(addr: SocketAddr, mask: &Ipv4Addr, actual: &Ipv4Addr) -> SocketAddr {
    match addr {
        SocketAddr::V4(addr) => {
            let virt = addr.ip().octets();
            let mask = mask.octets();
            let actual = actual.octets();
            SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(
                    (virt[0] & !mask[0]) | (actual[0] & mask[0]),
                    (virt[1] & !mask[1]) | (actual[1] & mask[1]),
                    (virt[2] & !mask[2]) | (actual[2] & mask[2]),
                    (virt[3] & !mask[3]) | (actual[3] & mask[3]),
                )),
                addr.port(),
            )
        }
        SocketAddr::V6(_) => {
            panic!("IpV6 is not supported!");
        }
    }
}

pub struct VirtualConnectionManager {
    inner: TcpConnectionManager,
    mask: Ipv4Addr,
    actual: Ipv4Addr,
}

impl ConnectionManager<TransportTcp> for VirtualConnectionManager {
    fn connection(
        &self,
        event_handler: Option<Sender<Envelope>>,
        error_handler: Option<Sender<Error>>,
        addr: SocketAddr,
    ) -> BoxFuture<Result<TransportTcp>> {
        self.inner.connection(
            event_handler,
            error_handler,
            rewrite(addr, &self.mask, &self.actual),
        )
    }
}

impl VirtualConnectionManager {
    async fn new(
        config: &VirtualClusterConfig,
        keyspace_holder: Arc<KeyspaceHolder>,
    ) -> Result<Self> {
        Ok(VirtualConnectionManager {
            inner: TcpConnectionManager::new(
                config.authenticator.clone(),
                keyspace_holder,
                Box::<ProtocolFrameEncodingFactory>::default(),
                Compression::None,
                DEFAULT_TRANSPORT_BUFFER_SIZE,
                true,
                config.version,
                #[cfg(feature = "http-proxy")]
                None,
            ),
            mask: config.mask,
            actual: config.actual,
        })
    }
}

impl GenericClusterConfig<TransportTcp, VirtualConnectionManager> for VirtualClusterConfig {
    fn create_manager(
        &self,
        keyspace_holder: Arc<KeyspaceHolder>,
    ) -> BoxFuture<Result<VirtualConnectionManager>> {
        // create a connection manager that points at the rewritten address so that's where it connects, but
        // then return a manager with the 'virtual' address for internal purposes.
        VirtualConnectionManager::new(self, keyspace_holder).boxed()
    }

    fn event_channel_capacity(&self) -> usize {
        EVENT_CHANNEL_CAPACITY
    }

    fn version(&self) -> Version {
        self.version
    }

    fn connection_pool_config(&self) -> ConnectionPoolConfig {
        Default::default()
    }
}
