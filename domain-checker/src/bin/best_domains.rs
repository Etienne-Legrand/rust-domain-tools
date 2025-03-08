use std::fs::File;
use std::collections::HashSet;
use anyhow::Result;
use regex::Regex;
use csv::{ReaderBuilder, Writer};

// Structure pour évaluer un domaine
#[derive(Debug)]
struct DomainScore {
    domain: String,
    score: i32,
}

// Critères d'évaluation d'un domaine
struct DomainEvaluator {
    good_words: HashSet<String>,
    consonant_pattern: Regex,
    vowels: HashSet<char>,
    bad_patterns: Vec<Regex>,
}

impl DomainEvaluator {
    // Initialiser les critères d'évaluation d'un domaine
    fn new() -> Self {
        let good_words: HashSet<String> = [
            "web", "vip", "dev", "box", "pix", "fun", "max", "zip", "top", "biz",
            "app", "tech", "lab", "hub", "pro", "net", "cloud", "smart", "digital",
            "code", "data", "ai", "io", "eco", "cyber", "meta", "crypto"
        ].iter().map(|s| s.to_string()).collect();

        let vowels: HashSet<char> = ['a', 'e', 'i', 'o', 'u', 'y'].into_iter().collect();

        let bad_patterns = vec![
            Regex::new(r"[bcdfghjklmnpqrstvwxz]{4,}").unwrap(), // 4+ consonnes consécutives
            Regex::new(r"[^a-zA-Z]").unwrap(), // autres caractères que des lettres
            Regex::new(r".{16,}").unwrap(), // Plus de 15 caractères
        ];

        Self {
            good_words,
            consonant_pattern: Regex::new(r"[bcdfghjklmnpqrstvwxz]{3,}").unwrap(),
            vowels,
            bad_patterns,
        }
    }

    // Évaluer un domaine en fonction des critères d'évaluation et retourner le score
    fn evaluate_domain(&self, domain: &str) -> Option<DomainScore> {
        let domain = domain.to_lowercase();
        let base_domain = domain.split('.').next()?;

        // Eliminer les domaines qui contiennent des mauvais patterns
        for pattern in &self.bad_patterns {
            if pattern.is_match(base_domain) {
                return None;
            }
        }

        // Calcul du score
        let mut score = 0i32;

        // Bonus pour les mots marketing
        if self.good_words.contains(base_domain) {
            score += 30;
        }

        // Bonus pour la présence de voyelles
        let vowel_count = base_domain.chars().filter(|c| self.vowels.contains(c)).count();
        score += (vowel_count as i32) * 5;

        // Malus pour les groupes de consonnes
        if self.consonant_pattern.is_match(base_domain) {
            score -= 15;
        }

        // Bonus pour l'alternance voyelles/consonnes
        let mut prev_is_vowel = None;
        let mut alternations = 0;
        for c in base_domain.chars() {
            let is_vowel = self.vowels.contains(&c);
            if let Some(prev) = prev_is_vowel {
                if prev != is_vowel {
                    alternations += 1;
                }
            }
            prev_is_vowel = Some(is_vowel);
        }
        score += alternations * 3;

        // Bonus pour les domaines courts (+5 par caractère en moins)
        if base_domain.len() <= 6 {
            score += (7 - base_domain.len() as i32) * 5;
        }

        Some(DomainScore {
            domain: domain.to_string(),
            score,
        })
    }
}

// Charger et évaluer les domaines
fn load_and_evaluate_domains(input_file: &str, evaluator: &DomainEvaluator) -> Result<Vec<DomainScore>> {
    // Ouvre le fichier d'entrée
    let file = File::open(input_file)?;
    
    // Configure le lecteur CSV
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let mut scores: Vec<DomainScore> = Vec::new();
    
    // Traite chaque domaine du fichier
    for result in reader.records() {
        let record = result?;
        if let Some(score) = evaluator.evaluate_domain(&record[0]) {
            scores.push(score);
        }
    }

    // Trie par score décroissant
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    Ok(scores)
}

// Sauvegarder les meilleurs domaines dans best_domains.csv
fn save_best_domains(domains: &[DomainScore], output_file: &str) -> Result<()> {
    let mut writer = Writer::from_path(output_file)?;
    
    // Écrire l'en-tête
    writer.write_record(&["domain", "score"])?;
    
    // Écrire les données
    for DomainScore { domain, score } in domains {
        writer.write_record(&[domain, &score.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

// Fonction principale pour charger, évaluer les domaines et sauvegarder les meilleurs domaines
fn main() -> Result<()> {
    let evaluator = DomainEvaluator::new();
    
    println!("Chargement et évaluation des domaines...");
    let scored_domains = load_and_evaluate_domains("domains.csv", &evaluator)?;
    
    println!("Sauvegarde des meilleurs domaines...");
    save_best_domains(&scored_domains, "best_domains.csv")?;
    
    println!("Top 10 des meilleurs domaines :");
    for (i, DomainScore { domain, score }) in scored_domains.iter().take(10).enumerate() {
        println!("{}. {} (score: {})", i + 1, domain, score);
    }
    
    println!("\nNombre total de domaines évalués : {}", scored_domains.len());
    println!("Résultats complets sauvegardés dans best_domains.csv");
    
    Ok(())
} 