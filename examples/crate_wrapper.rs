//! Wrapper utilisant zenth_protect comme crate
//! 
//! Ce wrapper utilise la crate zenth_protect pour analyser et sanitiser les fichiers.
//! Usage: cargo run --example crate_wrapper <fichier>

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process;

// Import depuis la crate zenth_protect
use zenth_protect::{
    sanitize_png, sanitize_jpeg, sanitize_pdf,
    sanitize_mp3, sanitize_mp4, sanitize_wav,
    detect_file_type,
    error::Error
};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: cargo run --example crate_wrapper <fichier>");
        eprintln!("Exemple: cargo run --example crate_wrapper document.pdf");
        process::exit(1);
    }
    
    let file_path = &args[1];
    
    match analyze_and_sanitize_file(file_path) {
        Ok(()) => {
            println!("✅ Fichier traité avec succès !");
        }
        Err(e) => {
            eprintln!("❌ Erreur: {}", e);
            process::exit(1);
        }
    }
}

fn analyze_and_sanitize_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Vérifier que le fichier existe
    if !Path::new(file_path).exists() {
        return Err(format!("Le fichier '{}' n'existe pas", file_path).into());
    }
    
    // Lire le fichier
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    println!("📁 Fichier: {}", file_path);
    println!("📏 Taille: {} bytes", buffer.len());
    
    // Détecter le type de fichier
    let file_type = detect_file_type(&buffer);
    println!("🔍 Type détecté: {}", file_type);
    
    // Vérifier si le format est supporté
    if file_type == "UNKNOWN" {
        return Err("Format de fichier non supporté".into());
    }
    
    // Sanitiser le fichier avec les vraies fonctions de la crate
    println!("🧹 Sanitisation en cours...");
    let clean_data = sanitize_file(&buffer, file_type)?;
    
    println!("✨ Sanitisation terminée !");
    println!("📏 Nouvelle taille: {} bytes", clean_data.len());
    
    // Générer le nom de fichier de sortie
    let output_path = get_output_path(file_path);
    println!("💾 Fichier de sortie: {}", output_path);
    
    // Écrire le fichier nettoyé
    let mut output = File::create(&output_path)?;
    output.write_all(&clean_data)?;
    
    // Afficher les informations
    print_file_info(file_path, &output_path, buffer.len(), clean_data.len(), file_type);
    
    Ok(())
}

// Utilisation des vraies fonctions de zenth_protect
fn sanitize_file(data: &[u8], file_type: &str) -> Result<Vec<u8>, Error> {
    match file_type {
        "PNG" => sanitize_png(data),
        "JPEG" => sanitize_jpeg(data),
        "PDF" => sanitize_pdf(data),
        "MP3" => sanitize_mp3(data),
        "MP4" => sanitize_mp4(data),
        "WAV" => sanitize_wav(data),
        _ => Err(Error::InvalidSignature("UNSUPPORTED")),
    }
}

fn get_output_path(original: &str) -> String {
    if let Some(dot_pos) = original.rfind('.') {
        let (name, ext) = original.split_at(dot_pos);
        format!("{}_sanitized{}", name, ext)
    } else {
        format!("{}_sanitized", original)
    }
}

fn print_file_info(original: &str, output: &str, original_size: usize, clean_size: usize, file_type: &str) {
    let separator = "=".repeat(50);
    println!("\n{}", separator);
    println!("📊 RAPPORT D'ANALYSE");
    println!("{}", separator);
    
    println!("🔹 Fichier original: {}", original);
    println!("🔹 Fichier sécurisé: {}", output);
    println!("🔹 Type: {}", file_type);
    println!("🔹 Taille originale: {} bytes ({:.2} KB)", original_size, original_size as f64 / 1024.0);
    println!("🔹 Taille nettoyée: {} bytes ({:.2} KB)", clean_size, clean_size as f64 / 1024.0);
    
    if clean_size < original_size {
        let reduction = original_size - clean_size;
        let percentage = (reduction as f64 / original_size as f64) * 100.0;
        println!("🔹 Réduction: {} bytes ({:.1}%)", reduction, percentage);
        println!("🔹 Métadonnées supprimées ✅");
    } else if clean_size == original_size {
        println!("🔹 Aucune modification 📝");
    } else {
        let increase = clean_size - original_size;
        println!("🔹 Augmentation: {} bytes (reconstruction)", increase);
    }
    
    println!("{}", separator);
    
    // Informations spécifiques au type
    match file_type {
        "PNG" => println!("🖼️  Image PNG - Chunks validés et métadonnées nettoyées"),
        "JPEG" => println!("📸 Image JPEG - En-têtes validés et EXIF supprimé"),
        "PDF" => println!("📄 Document PDF - Objets validés et scripts supprimés"),
        "MP3" => println!("🎵 Audio MP3 - Tags ID3 nettoyés"),
        "MP4" => println!("🎥 Vidéo MP4 - Atomes validés et métadonnées supprimées"),
        "WAV" => println!("🔊 Audio WAV - En-têtes validés"),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_png() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52
        ];
        assert_eq!(detect_file_type(&png_data), "PNG");
    }
    
    #[test]
    fn test_detect_jpeg() {
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(detect_file_type(&jpeg_data), "JPEG");
    }
    
    #[test]
    fn test_detect_pdf() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog";
        assert_eq!(detect_file_type(pdf_data), "PDF");
    }
    
    #[test]
    fn test_detect_mp3() {
        let mp3_data = b"ID3\x04\x00\x00\x00\x00\x00\x00";
        assert_eq!(detect_file_type(mp3_data), "MP3");
    }
    
    #[test]
    fn test_detect_unknown() {
        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(detect_file_type(&unknown_data), "UNKNOWN");
    }
    
    #[test]
    fn test_output_path_generation() {
        assert_eq!(get_output_path("test.png"), "test_sanitized.png");
        assert_eq!(get_output_path("document.pdf"), "document_sanitized.pdf");
        assert_eq!(get_output_path("noext"), "noext_sanitized");
    }
}
