mod task;
mod error;

// Startup application params
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");


const DEFAULT_STATEBOARD_PORT: u16 = 4401;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    task::run_v2()?;
    Ok(())
}
