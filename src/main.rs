mod error;
mod task;

// Startup application params
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

const DEFAULT_STATEBOARD_PORT: u16 = 4401;
const DEFAULT_HTTP_PORT: u16 = 8081;
const DEFAULT_BINARY_PORT: u16 = 3031;

const DEFAULT_CFG_NAME: &str = "cluster.genin.yml";
const DEFAULT_CFG: &[u8] = include_bytes!("../configs/cluster.genin.yml");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    task::run_v2()?;
    Ok(())
}
