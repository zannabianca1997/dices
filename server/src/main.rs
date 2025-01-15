use clap::Parser;
use dices_server::{CliArgs, MainError};

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let args = CliArgs::parse();
    dices_server::main(args).await.map(drop)
}
