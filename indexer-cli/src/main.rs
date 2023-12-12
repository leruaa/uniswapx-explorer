use clap::Parser;
use cli::Cli;
use dotenv::dotenv;
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

mod cli;


// 0x6880c8341a8e00e9c0b4a4fd2a2934980cd0851d2fc5a86d3ed09c55fd380912
// eyJjaGFpbklkIjoxLCJjcmVhdGVkQXQiOjE2NzkyNDA0MzksIm9yZGVySGFzaCI6IjB4Njg4MGM4MzQxYThlMDBlOWMwYjRhNGZkMmEyOTM0OTgwY2QwODUxZDJmYzVhODZkM2VkMDljNTVmZDM4MDkxMiJ9

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();

    let stdout_log = tracing_subscriber::fmt::layer()
        .compact()
        .with_target(false)
        .with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        );

    tracing_subscriber::registry().with(stdout_log).init();

    indexer::start(cli.eth_http_rpc).await.unwrap();
}
