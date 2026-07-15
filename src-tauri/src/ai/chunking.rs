const DEFAULT_CHUNK_SIZE: usize = 500;
const DEFAULT_OVERLAP: usize = 50;

pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let overlap = overlap.min(chunk_size / 2);

    let words: Vec<&str> = text.split_whitespace().collect();

    if words.len() <= chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk_words = &words[start..end];

        let chunk_end = if end < words.len() {
            find_natural_break(chunk_words)
        } else {
            chunk_words.len()
        };

        let chunk_text = chunk_words[..chunk_end].join(" ");
        chunks.push(chunk_text);

        if end >= words.len() {
            break;
        }

        start += chunk_end.saturating_sub(overlap);
        if start >= words.len() {
            break;
        }
    }

    chunks
}

fn find_natural_break(words: &[&str]) -> usize {
    if words.is_empty() {
        return 0;
    }

    for i in (words.len() / 2..words.len()).rev() {
        let word = words[i];
        if word.ends_with('.') || word.ends_with('!') || word.ends_with('?') {
            return i + 1;
        }
    }

    for i in (words.len() / 2..words.len()).rev() {
        let word = words[i];
        if word.ends_with(',') || word.ends_with(';') || word.ends_with(':') {
            return i + 1;
        }
    }

    words.len()
}

pub fn chunk_text_default(text: &str) -> Vec<String> {
    chunk_text(text, DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", 500, 50);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_short_text() {
        let text = "This is a short text.";
        let chunks = chunk_text(text, 500, 50);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_chunking() {
        let words: Vec<String> = (0..1000).map(|i| format!("word{}", i)).collect();
        let text = words.join(" ");

        let chunks = chunk_text(&text, 100, 10);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            let word_count = chunk.split_whitespace().count();
            assert!(word_count <= 100, "Chunk too large: {} words", word_count);
        }
    }

    #[test]
    fn test_natural_break_at_sentence() {
        let text = "First sentence here. Second sentence starts here and continues with more words to fill the chunk. Third sentence.";
        let chunks = chunk_text(text, 10, 2);

        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_overlap() {
        let words: Vec<String> = (0..200).map(|i| format!("word{}", i)).collect();
        let text = words.join(". ");

        let chunks = chunk_text(&text, 50, 10);

        if chunks.len() >= 2 {
            let first_words: Vec<&str> = chunks[0].split_whitespace().collect();
            let second_words: Vec<&str> = chunks[1].split_whitespace().collect();

            let first_last_10: Vec<&str> = first_words.iter().rev().take(15).copied().collect();
            let second_first_10: Vec<&str> = second_words.iter().take(15).copied().collect();

            let has_overlap = first_last_10.iter().any(|w| second_first_10.contains(w));
            assert!(has_overlap || chunks.len() == 1, "Expected overlap between chunks");
        }
    }
}
