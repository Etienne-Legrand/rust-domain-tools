class DomainGenerator {
  constructor() {
    this.alphabet = "abcdefghijklmnopqrstuvwxyz";
  }

  /**
   * Génère des combinaisons de domaines de manière asynchrone et progressive
   * @param {number} length - Longueur des domaines
   * @param {string[]} tlds - Liste des extensions de domaine
   * @returns {AsyncGenerator} Générateur asynchrone de domaines
   */
  async *generateDomainCombinations(length, tlds) {
    const totalCombinations = Math.pow(this.alphabet.length, length);
    const totalDomains = totalCombinations * tlds.length;
    let currentProgress = 0; // Compteur pour la progression globale

    // Générateur progressif sans limite stricte
    for (let i = 0; i < totalCombinations; i++) {
      let domain = this.convertToDomain(i, length);
      for (let tld of tlds) {
        // Calcul de la progression globale
        currentProgress++;

        // Mettre à jour la progression
        yield {
          domain: `${domain}.${tld}`,
          progress: (currentProgress / totalDomains) * 100, // Calcul de la progression globale
        };

        // Petit délai pour permettre au navigateur de respirer
        if (i % 1000 === 0) {
          await new Promise((resolve) => setTimeout(resolve, 0));
        }
      }
    }
  }

  /**
   * Convertit un index numérique en chaîne de caractères de domaine
   * @param {number} index - Index à convertir
   * @param {number} length - Longueur du domaine
   * @returns {string} Domaine généré
   */
  convertToDomain(index, length) {
    let domain = "";
    for (let i = 0; i < length; i++) {
      domain = this.alphabet[index % this.alphabet.length] + domain;
      index = Math.floor(index / this.alphabet.length);
    }
    return domain;
  }

  /**
   * Crée un fichier ZIP avec les domaines générés de manière progressive
   * @param {AsyncGenerator} domainGenerator - Générateur de domaines
   * @param {number} maxRowsPerFile - Nombre maximum de lignes par fichier
   * @param {Function} progressCallback - Fonction de suivi de progression
   * @returns {Promise<void>}
   */
  async createZipFile(
    domainGenerator,
    maxRowsPerFile = 500,
    progressCallback = null
  ) {
    const zip = new JSZip();
    let fileCount = 1;
    let currentFile = [];
    let totalDomainsGenerated = 0;

    for await (let item of domainGenerator) {
      currentFile.push(item.domain);
      totalDomainsGenerated++;

      // Mettre à jour la progression
      if (progressCallback) {
        progressCallback({
          progress: item.progress,
          filesGenerated: fileCount,
          totalDomainsGenerated,
        });
      }

      // Créer un nouveau fichier quand la limite est atteinte
      if (currentFile.length === maxRowsPerFile) {
        const fileName = `domains_part_${fileCount}.csv`;
        zip.file(fileName, currentFile.join("\n"));
        currentFile = [];
        fileCount++;
      }

      // Petit délai pour ne pas bloquer l'UI
      if (totalDomainsGenerated % 1000 === 0) {
        await new Promise((resolve) => setTimeout(resolve, 0));
      }
    }

    // Ajouter le dernier fichier s'il reste des domaines
    if (currentFile.length > 0) {
      const fileName = `domains_part_${fileCount}.csv`;
      zip.file(fileName, currentFile.join("\n"));
    }

    // Générer et télécharger le ZIP
    const zipContent = await zip.generateAsync({ type: "blob" });
    saveAs(zipContent, "domains.zip");

    return totalDomainsGenerated;
  }
}

/**
 * Fonction de démarrage pour générer les domaines
 */
function startGeneration() {
  try {
    // Récupérer les valeurs des inputs
    const lengthInput = document.getElementById("length");
    const tldsInput = document.getElementById("tlds");
    const progressDisplay = document.getElementById("progressDisplay");
    const progressBar = document.getElementById("progressBar");

    const length = parseInt(lengthInput.value);
    const tlds = tldsInput.value
      .split(",")
      .map((tld) => tld.trim())
      .filter((tld) => tld.length > 0);

    // Validation des entrées
    if (isNaN(length) || length < 1 || length > 6) {
      alert("Veuillez saisir une longueur valide entre 1 et 6 caractères.");
      return;
    }

    if (tlds.length === 0) {
      alert("Veuillez saisir au moins un TLD.");
      return;
    }

    // Désactiver le bouton pendant la génération
    const generateButton = document.querySelector("button");
    generateButton.disabled = true;
    generateButton.textContent = "Génération en cours...";

    // Réinitialiser la progression
    progressDisplay.textContent = "Initialisation de la génération...";
    progressBar.style.width = "0%";
    progressBar.textContent = "0%";

    // Instance du générateur
    const generator = new DomainGenerator();

    // Créer le générateur de domaines
    const domainGenerator = generator.generateDomainCombinations(length, tlds);

    // Fonction de suivi de progression
    const updateProgress = (progress) => {
      progressDisplay.textContent =
        `Fichiers générés : ${progress.filesGenerated} | ` +
        `Domaines créés : ${progress.totalDomainsGenerated.toLocaleString()}`;

      // Mettre à jour la barre de progression
      progressBar.style.width = `${Math.min(progress.progress, 100)}%`;
      progressBar.textContent = `${Math.round(progress.progress)}%`;
    };

    // Créer et télécharger le ZIP
    generator
      .createZipFile(domainGenerator, 50000, updateProgress)
      .then((totalDomains) => {
        // Réactiver le bouton
        generateButton.disabled = false;
        generateButton.textContent = "Générer et télécharger le ZIP";

        progressDisplay.textContent = `Génération terminée ! Total de domaines : ${totalDomains.toLocaleString()}`;
        progressBar.style.width = "100%";
        progressBar.textContent = "100%";
      })
      .catch((error) => {
        console.error("Erreur lors de la génération :", error);
        alert(`Erreur : ${error.message}`);

        // Réactiver le bouton en cas d'erreur
        generateButton.disabled = false;
        generateButton.textContent = "Générer et télécharger le ZIP";
        progressDisplay.textContent = "Erreur lors de la génération";
        progressBar.style.width = "0%";
        progressBar.textContent = "0%";
      });
  } catch (error) {
    console.error("Erreur inattendue :", error);
    alert(`Une erreur inattendue s'est produite : ${error.message}`);
  }
}
