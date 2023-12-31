use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "TradingPost",
    version = "0.1.0",
    about = "Trading post microservice"
)]
pub struct CliOptions {
    #[structopt(
        short = "h",
        long = "host",
        help = "The used IP for a server",
        default_value = "127.0.0.1"
    )]
    pub host: String,
    #[structopt(
        short = "p",
        long = "port",
        help = "The listened port",
        default_value = "8000"
    )]
    pub port: u16,

    #[structopt(
        long = "cassandra-nodes",
        help = "Cassandra nodes",
        default_value = "127.0.0.1:9042",
        env = "CASSANDRA_NODES"
    )]
    pub cassandra_nodes: String,

    #[structopt(
        long = "cassandra-user",
        help = "Cassandra user",
        default_value = "cassandra",
        env = "CASSANDRA_USERNAME"
    )]
    pub cassandra_user: String,

    #[structopt(
        long = "cassandra-password",
        help = "Cassandra user's password",
        default_value = "cassandra",
        env = "CASSANDRA_PASSWORD"
    )]
    pub cassandra_password: String,
}
