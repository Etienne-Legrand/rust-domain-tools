use anyhow::Result;
use back_check_domain::all_checker::AllDomainChecker;

#[tokio::main]
async fn main() -> Result<()> {
    let checker = AllDomainChecker::new();
    checker.process_domains("domains.csv", "potential_domains.csv").await?;
    Ok(())
} 