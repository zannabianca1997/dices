#![feature(error_reporter)]
use std::error::Report;

use clap::Parser;
use dices_server::{CliArgs, MainError};

#[tokio::main]
async fn main() -> Result<(), Report<MainError>> {
    let args = CliArgs::parse();
    dices_server::main(args)
        .await
        .map_err(|err| Report::new(err).pretty(true))
        .map(drop)
}
