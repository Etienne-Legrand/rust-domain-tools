use anyhow::Result;
use back_check_domain::fr_checker::FrDomainChecker;

#[tokio::main]
async fn main() -> Result<()> {
    let checker = FrDomainChecker::new().await?;
    checker.process_domains("domains.csv", "available_fr_domains.csv").await?;
    Ok(())
} 