use std::path::Path;

pub fn parse_pdf(path: &Path) -> Result<String, String> {
    pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract text from PDF: {}", e))
        .map(|text| {
            let cleaned = clean_pdf_text(&text);
            if cleaned.len() < 50 {
                eprintln!("Warning: PDF extraction yielded minimal text - may be a scanned document");
            }
            cleaned
        })
}

fn clean_pdf_text(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    lines.dedup();

    let mut result = String::new();
    let mut prev_was_paragraph = false;

    for line in lines {
        let is_paragraph_start = line.chars().next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        if is_paragraph_start && prev_was_paragraph {
            result.push_str("\n\n");
        } else if !result.is_empty() {
            result.push(' ');
        }

        result.push_str(line);
        prev_was_paragraph = line.ends_with('.') || line.ends_with('!') || line.ends_with('?');
    }

    result
}

pub fn is_likely_scanned_pdf(path: &Path) -> Result<bool, String> {
    match parse_pdf(path) {
        Ok(text) => Ok(text.trim().len() < 100),
        Err(_) => Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_pdf_text() {
        let input = "  Hello world.  \n\n\n  Another line.  \n  ";
        let cleaned = clean_pdf_text(input);
        assert!(cleaned.contains("Hello world."));
        assert!(cleaned.contains("Another line."));
    }

    #[test]
    fn test_clean_pdf_text_dedup() {
        let input = "Line 1\nLine 1\nLine 2";
        let cleaned = clean_pdf_text(input);
        assert_eq!(cleaned.matches("Line 1").count(), 1);
    }
}
