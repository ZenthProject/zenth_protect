/// Représente les métadonnées d'un PDF extraites de l'objet /Info
/// 
/// SÉCURITÉ : Ces informations peuvent révéler l'identité et les habitudes de l'utilisateur.
/// Toutes les métadonnées doivent être supprimées lors de la sanitization.
/// 
/// # Champs standards PDF
/// 
/// Les champs suivants sont définis dans la spécification PDF (ISO 32000) :
/// - `/Author` : Nom de la personne qui a créé le document
/// - `/Creator` : Nom du logiciel qui a créé le document original
/// - `/Producer` : Nom du logiciel qui a converti le document en PDF
/// - `/CreationDate` : Date et heure de création (format : D:YYYYMMDDHHmmSSOHH'mm)
/// - `/ModDate` : Date et heure de dernière modification
/// - `/Title` : Titre du document
/// - `/Subject` : Sujet du document
/// - `/Keywords` : Mots-clés associés au document
/// 
/// # Champs personnalisés
/// 
/// Certains logiciels ajoutent des champs custom qui peuvent être dangereux :
/// - `/GPS` : Coordonnées GPS (localisation)
/// - `/Company` : Nom de l'entreprise
/// - `/Manager` : Nom du responsable
/// - Tout autre champ commençant par `/`
#[derive(Debug, Clone, PartialEq)]
pub struct PdfMetadata {
    /// Auteur du document (/Author)
    /// Exemple : "John Doe"
    pub author: Option<String>,
    
    /// Logiciel créateur (/Creator)
    /// Exemple : "Microsoft Word", "LibreOffice Writer"
    pub creator: Option<String>,
    
    /// Producteur PDF (/Producer)
    /// Exemple : "Adobe PDF Library 15.0"
    pub producer: Option<String>,
    
    /// Date de création (/CreationDate)
    /// Format PDF : D:YYYYMMDDHHmmSSOHH'mm
    /// Exemple : "D:20240315120000+01'00"
    pub creation_date: Option<String>,
    
    /// Date de modification (/ModDate)
    /// Format identique à creation_date
    pub mod_date: Option<String>,
    
    /// Titre du document (/Title)
    /// Exemple : "Rapport confidentiel"
    pub title: Option<String>,
    
    /// Sujet du document (/Subject)
    /// Exemple : "Analyse financière Q4 2024"
    pub subject: Option<String>,
    
    /// Mots-clés (/Keywords)
    /// Exemple : "confidentiel, finance, rapport"
    pub keywords: Option<String>,
    
    /// Champs personnalisés (ex: /GPS, /Company, etc.)
    /// Stockés comme paires (clé, valeur)
    /// Exemple : [("GPS", "48.8566, 2.3522"), ("Company", "Acme Corp")]
    pub custom: Vec<(String, String)>,
}

impl PdfMetadata {
    /// Crée une structure vide (aucune métadonnée)
    /// 
    /// # Exemple
    /// 
    /// ```
    /// # use zenth_protect::pdf::metadata_pdf::PdfMetadata;
    /// let metadata = PdfMetadata::new();
    /// assert!(!metadata.has_metadata());
    /// ```
    pub fn new() -> Self {
        Self {
            author: None,
            creator: None,
            producer: None,
            creation_date: None,
            mod_date: None,
            title: None,
            subject: None,
            keywords: None,
            custom: Vec::new(),
        }
    }
    
    /// Vérifie si des métadonnées sont présentes
    /// 
    /// Retourne `true` si AU MOINS un champ contient des données
    /// 
    /// # Exemple
    /// 
    /// ```
    /// # use zenth_protect::pdf::metadata_pdf::PdfMetadata;
    /// let mut metadata = PdfMetadata::new();
    /// assert!(!metadata.has_metadata());
    ///
    /// metadata.author = Some("John Doe".to_string());
    /// assert!(metadata.has_metadata());
    /// ```
    pub fn has_metadata(&self) -> bool {
        self.author.is_some()
            || self.creator.is_some()
            || self.producer.is_some()
            || self.creation_date.is_some()
            || self.mod_date.is_some()
            || self.title.is_some()
            || self.subject.is_some()
            || self.keywords.is_some()
            || !self.custom.is_empty()
    }
    
    /// Compte le nombre total de champs remplis
    /// 
    /// Utile pour afficher des statistiques sur la quantité de métadonnées
    /// 
    /// # Exemple
    /// 
    /// ```
    /// # use zenth_protect::pdf::metadata_pdf::PdfMetadata;
    /// let mut metadata = PdfMetadata::new();
    /// assert_eq!(metadata.field_count(), 0);
    /// 
    /// metadata.author = Some("John".to_string());
    /// metadata.creator = Some("Word".to_string());
    /// metadata.custom.push(("GPS".to_string(), "48.8566, 2.3522".to_string()));
    /// 
    /// assert_eq!(metadata.field_count(), 3);
    /// ```
    pub fn field_count(&self) -> usize {
        let mut count = 0;
        
        // Compter les champs standards
        if self.author.is_some() { count += 1; }
        if self.creator.is_some() { count += 1; }
        if self.producer.is_some() { count += 1; }
        if self.creation_date.is_some() { count += 1; }
        if self.mod_date.is_some() { count += 1; }
        if self.title.is_some() { count += 1; }
        if self.subject.is_some() { count += 1; }
        if self.keywords.is_some() { count += 1; }
        
        // Ajouter les champs custom
        count + self.custom.len()
    }
}

/// Implémentation du trait Default
/// 
/// Permet d'utiliser `PdfMetadata::default()` au lieu de `PdfMetadata::new()`
/// C'est une convention Rust pour les types qui ont une valeur "par défaut"
impl Default for PdfMetadata {
    fn default() -> Self {
        Self::new()
    }
}
