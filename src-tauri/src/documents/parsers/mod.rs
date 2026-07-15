pub mod pdf;
pub mod xlsx;

use std::path::Path;

pub fn parse_document(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "xlsx" | "xls" => xlsx::parse_xlsx(path),
        "pdf" => pdf::parse_pdf(path),
        "txt" | "md" | "markdown" => {
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read text file: {}", e))
        }
        "json" => {
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read JSON file: {}", e))
        }
        "csv" => parse_csv(path),
        _ => Err(format!("Unsupported file type: .{}", extension)),
    }
}

fn parse_csv(path: &Path) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read CSV file: {}", e))?;

    let mut output = String::new();
    let mut lines = content.lines();

    if let Some(header) = lines.next() {
        let headers: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
        output.push_str("| ");
        output.push_str(&headers.join(" | "));
        output.push_str(" |\n");

        output.push_str("|");
        for _ in &headers {
            output.push_str(" --- |");
        }
        output.push('\n');
    }

    for line in lines.take(10_000) {
        let cells: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        output.push_str("| ");
        output.push_str(&cells.join(" | "));
        output.push_str(" |\n");
    }

    Ok(output)
}

pub fn get_supported_extensions() -> Vec<&'static str> {
    vec!["xlsx", "xls", "pdf", "txt", "md", "markdown", "json", "csv"]
}

pub fn is_supported_extension(ext: &str) -> bool {
    let ext_lower = ext.to_lowercase();
    get_supported_extensions().contains(&ext_lower.as_str())
}
