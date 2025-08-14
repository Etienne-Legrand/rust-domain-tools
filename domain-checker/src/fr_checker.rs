use std::fs::File;
use std::time::Duration;
use std::thread;
use anyhow::{Result, Context};
use csv::{ReaderBuilder, Writer};
use scraper::{Html, Selector};

// Structure principale qui gère la vérification des domaines .fr
pub struct FrDomainChecker {
    security_token: String,
}

impl FrDomainChecker {
    // Crée une nouvelle instance en récupérant le token de sécurité
    pub async fn new() -> Result<Self> {
        let security_token = Self::fetch_security_token().await?;
        Ok(Self { security_token })
    }

    // Récupère le token de sécurité depuis le site de l'AFNIC
    async fn fetch_security_token() -> Result<String> {
        let client = reqwest::Client::new();
        // Fait une requête GET sur la page WHOIS
        let response = client
            .get("https://www.afnic.fr/en/domain-names-and-support/everything-there-is-to-know-about-domain-names/find-a-domain-name-or-a-holder-using-whois/")
            .send()
            .await?
            .text()
            .await?;

        // Extrait le token de sécurité du HTML
        let document = Html::parse_document(&response);
        let selector = Selector::parse("#security").unwrap();
        
        let security_token = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("value"))
            .context("Security token non trouvé")?
            .to_string();

        Ok(security_token)
    }

    // Vérifie si un domaine spécifique est disponible
    pub async fn check_domain(&self, domain: &str) -> Result<bool> {
        // Vérifie que c'est bien un domaine .fr
        if !domain.ends_with(".fr") {
            return Ok(false);
        }

        let client = reqwest::Client::new();
        
        // Sépare le nom de domaine de l'extension
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() != 2 {
            return Ok(false);
        }
        
        // Prépare les paramètres pour la requête WHOIS
        let params = [
            ("action", "ajax_get_whois_info"),
            ("d", parts[0]),
            ("tld", parts[1]),
            ("lang", "en"),
            ("security", &self.security_token),
            ("name", "1"),
        ];
        
        // Envoie la requête à l'API AFNIC
        let response = client
            .post("https://www.afnic.fr/wp-admin/admin-ajax.php")
            .form(&params)
            .send()
            .await?;

        let body = response.text().await?;
        
        if body == "0" {
            return Ok(false);
        }
        
        Ok(body.contains("available"))
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
            
            if !domain.ends_with(".fr") {
                continue;
            }

            // Vérifie la disponibilité et enregistre si disponible
            match self.check_domain(domain).await {
                Ok(available) => {
                    if available {
                        writer.write_record(&[domain])?;
                        println!("{}: DISPONIBLE", domain);
                    } else {
                        println!("{}: NON DISPONIBLE", domain);
                    }
                }
                Err(e) => println!("Erreur pour {}: {}", domain, e),
            }
            
            // Pause d'une seconde pour ne pas surcharger l'API
            thread::sleep(Duration::from_secs(1));
        }

        writer.flush()?;
        println!("\nRésultats complets sauvegardés dans {}", output_file);
        Ok(())
    }
} 