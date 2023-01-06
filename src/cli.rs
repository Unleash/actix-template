use clap::{Args, Parser};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct SSLOptions {
    /// Your server key
    #[clap(env, short = 'k', long)]
    pub server_key: Option<PathBuf>,

    /// Your server certificate
    #[clap(env, short = 'c', long)]
    pub server_cert: Option<PathBuf>,

    /// Which port should we bind the SSL server to
    #[clap(env, short, long, default_value_t = 1837)]
    pub ssl_port: u16,
}

#[derive(Parser, Debug, Clone)]
pub struct ServerArgs {
    /// Which port should we mount the server to
    #[clap(env, short, long, default_value_t = 1337)]
    pub port: u16,

    /// Which interface should we mount the server to
    #[clap(env, short, long, default_value = "0.0.0.0")]
    pub interface: String,

    #[clap(flatten)]
    pub ssl: SSLOptions,
}
