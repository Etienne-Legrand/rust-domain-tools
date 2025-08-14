use anyhow::Result;
use domain_checker::all_checker::AllDomainChecker;

#[tokio::main]
async fn main() -> Result<()> {
    let checker = AllDomainChecker::new();
    checker.process_domains("domains.csv", "potential_domains.csv").await?;
    Ok(())
} 