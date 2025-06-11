#[derive(clap::Parser)]
#[command(version, about = "Test runner")]
pub struct Cli {
    #[command(subcommand)]
    pub test_type: TestType,
}

#[derive(clap::Subcommand)]
pub enum TestType {
    /// End-to-end tests
    E2e {
        #[arg(long)]
        operation: Option<Operation>,
    },
    /// Concurrency tests
    Concurrency {
        #[arg(long)]
        operation: Option<Operation>,
        #[arg(long, value_name = "N")]
        count: Option<u32>,
    },
    /// Stress tests
    Stress {
        #[arg(long)]
        operation: Option<Operation>,
        #[arg(long, value_name = "N")]
        count: Option<u32>,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Operation {
    Melt,
    Mint,
    Send,
    Receive,
}
fn main() {
    println!("Hello, world!");
}
