use anyhow::Result;
use domain_checker::fr_checker::FrDomainChecker;

#[tokio::main]
async fn main() -> Result<()> {
    let checker = FrDomainChecker::new().await?;
    checker.process_domains("domains.csv", "available_fr_domains.csv").await?;
    Ok(())
} 