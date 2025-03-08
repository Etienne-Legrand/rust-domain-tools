use std::fs::File;
use std::time::Duration;
use anyhow::{Result, Context};
use csv::{ReaderBuilder, Writer};

// Structure pour vérifier tous les types de domaines
pub struct AllDomainChecker {
    client: reqwest::Client,
}

impl AllDomainChecker {
    // Crée une nouvelle instance
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    // Vérifie si un domaine est potentiellement disponible
    pub async fn check_domain(&self, domain: &str) -> Result<bool> {
        // Ajoute "https://" si nécessaire
        let url = if domain.starts_with("http") {
            domain.to_string()
        } else {
            format!("https://{}", domain)
        };

        // Essaie de faire une requête GET sur le domaine
        match self.client.get(&url).send().await {
            Ok(_) => Ok(false),
            Err(_) => Ok(true)
        }
    }

    // Traite tous les domaines du fichier CSV
    pub async fn process_domains(&self, input_file: &str, output_file: &str) -> Result<()> {
        // Ouvre le fichier d'entrée
        let file = File::open(input_file)
            .context("Impossible d'ouvrir le fichier d'entrée")?;
        
        // Configure le lecteur CSV
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        // Prépare le fichier de sortie CSV
        let mut writer = Writer::from_path(output_file)
            .context("Impossible de créer le fichier de sortie")?;

        // Traite chaque domaine du fichier
        for result in reader.records() {
            let record = result?;
            let domain = &record[0];

            // Vérifie la disponibilité et enregistre si potentiellement disponible
            match self.check_domain(domain).await {
                Ok(available) => {
                    if available {
                        writer.write_record(&[domain])?;
                        println!("{}: POTENTIELLEMENT DISPONIBLE", domain);
                    } else {
                        println!("{}: NON DISPONIBLE", domain);
                    }
                }
                Err(e) => println!("Erreur pour {}: {}", domain, e),
            }
        }

        writer.flush()?;
        Ok(())
    }
} 