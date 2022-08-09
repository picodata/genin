#![feature(generic_associated_types)]

mod task;
mod error;

// Application params
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    task::run()?;
    Ok(())
}
