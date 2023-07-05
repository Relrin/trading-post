use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "TradingPost",
    version = "0.1.0",
    about = "Trading post microservice",
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
}