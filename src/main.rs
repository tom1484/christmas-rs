#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod action;
mod app;
mod components;
mod config;
mod constants;
mod pages;
mod tui;
mod utils;

use color_eyre::eyre::Result;

use crate::{
    app::App,
    utils::{initialize_logging, initialize_panic_handler},
};

async fn tokio_main() -> Result<()> {
    initialize_logging()?;

    initialize_panic_handler()?;

    let mut app = App::new(60.0, 60.0)?;
    app.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = tokio_main().await {
        eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
        Err(e)
    } else {
        Ok(())
    }
}
