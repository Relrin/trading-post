use std::sync::Arc;
use cdrs_tokio::authenticators::NoneAuthenticatorProvider;

use cdrs_tokio::cluster::session::{Session, SessionBuilder, TcpSessionBuilder};
use cdrs_tokio::cluster::{NodeTcpConfigBuilder, TcpConnectionManager};
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;
use cdrs_tokio::transport::TransportTcp;

use crate::cli::CliOptions;

#[derive(Clone)]
pub struct CassandraSession {
    inner: Arc<Session<
        TransportTcp,
        TcpConnectionManager,
        RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>,
    >>
}

impl CassandraSession {
    pub async fn new(opts: &CliOptions) -> Self {
        let cluster_config = NodeTcpConfigBuilder::new()
            .with_contact_point("127.0.0.1:9042".into())
            .with_authenticator_provider(Arc::new(NoneAuthenticatorProvider))
            .build()
            .await
            .unwrap();
        let lb = RoundRobinLoadBalancingStrategy::new();
        let session= TcpSessionBuilder::new(lb, cluster_config)
            .build()
            .await
            .unwrap();

        Self {
            inner: Arc::new(session)
        }
    }
}