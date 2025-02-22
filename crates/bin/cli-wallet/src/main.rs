use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, value_hint(ValueHint::FilePath))]
    db_path: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Mint new tokens
    Mint {
        #[arg(long, short, value_hint(ValueHint::Url))]
        node_url: String,
        #[arg(long, short)]
        amount: u64,
        #[arg(long, short)]
        unit: String,
    },

    /// Melt (burn) existing tokens
    Melt {
        /// Amount of tokens to melt
        #[arg(long)]
        amount: u64,

        /// Address to melt from
        #[arg(long)]
        from: String,
    },

    /// Swap tokens
    Swap {
        /// Amount of tokens to swap
        #[arg(long)]
        amount: u64,

        /// Token to swap from
        #[arg(long)]
        from_token: String,

        /// Token to swap to
        #[arg(long)]
        to_token: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Mint {
            node_url,
            amount,
            unit,
        } => {
            println!("Asking {} to mint {} {}", node_url, amount, unit);
            // Add mint logic here
            wallet::mint().unwrap();
        }
        Commands::Melt { amount, from } => {
            println!("Melting {} tokens from {}", amount, from);
            // Add melt logic here
        }
        Commands::Swap {
            amount,
            from_token,
            to_token,
        } => {
            println!(
                "Swapping {} tokens from {} to {}",
                amount, from_token, to_token
            );
            // Add swap logic here
        }
    }
}
