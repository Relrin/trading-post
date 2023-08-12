use std::sync::Arc;

use cdrs_tokio::authenticators::{StaticPasswordAuthenticatorProvider};
use cdrs_tokio::cluster::session::{Session, SessionBuilder, TcpSessionBuilder};
use cdrs_tokio::cluster::{NodeTcpConfigBuilder, TcpConnectionManager};
use cdrs_tokio::load_balancing::RoundRobinLoadBalancingStrategy;
use cdrs_tokio::transport::TransportTcp;

use crate::cli::CliOptions;

pub type CassandraSession = Arc<Session<
    TransportTcp,
    TcpConnectionManager,
    RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>,
>>;

pub async fn create_cassandra_session(opts: &CliOptions) -> CassandraSession {
    let address = format!("{0}:{1}", opts.cassandra_host, opts.cassandra_port);
    let auth = StaticPasswordAuthenticatorProvider::new(
        opts.cassandra_user.clone(),
        opts.cassandra_password.clone(),
    );
    let cluster_config = NodeTcpConfigBuilder::new()
        .with_contact_point(address.into())
        .with_authenticator_provider(Arc::new(auth))
        .build()
        .await
        .unwrap();
    let lb = RoundRobinLoadBalancingStrategy::new();
    let session= TcpSessionBuilder::new(lb, cluster_config)
        .build()
        .await
        .unwrap();

    Arc::new(session)
}