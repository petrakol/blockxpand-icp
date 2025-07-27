use aggregator::TokenTotal;
use anyhow::Result;
use bx_core::Holding;
use candid::{Decode, Encode, Principal};
use clap::{Parser, Subcommand};
use ic_agent::Agent;

#[derive(Parser)]
struct Cli {
    /// Aggregator canister ID
    #[arg(long)]
    canister: String,
    /// Replica URL
    #[arg(long, default_value = "http://localhost:4943")]
    url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch holdings for a principal
    Holdings { principal: String },
    /// Fetch summary for a principal
    Summary { principal: String },
}

async fn get_agent(url: &str) -> Agent {
    let agent = Agent::builder().with_url(url).build().unwrap();
    if url.contains("localhost") {
        let _ = agent.fetch_root_key().await;
    }
    agent
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cid = Principal::from_text(cli.canister)?;
    let agent = get_agent(&cli.url).await;
    match cli.command {
        Commands::Holdings { principal } => {
            let p = Principal::from_text(principal)?;
            let arg = Encode!(&p)?;
            let bytes = agent
                .query(&cid, "get_holdings")
                .with_arg(arg)
                .call()
                .await?;
            let holdings: Vec<Holding> = Decode!(&bytes, Vec<Holding>)?;
            println!("{}", serde_json::to_string_pretty(&holdings)?);
        }
        Commands::Summary { principal } => {
            let p = Principal::from_text(principal)?;
            let arg = Encode!(&p)?;
            let bytes = agent
                .query(&cid, "get_summary")
                .with_arg(arg)
                .call()
                .await?;
            let summary: Vec<TokenTotal> = Decode!(&bytes, Vec<TokenTotal>)?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
    }
    Ok(())
}
