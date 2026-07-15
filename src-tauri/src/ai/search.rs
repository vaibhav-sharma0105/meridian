use crate::models::document::SearchResult;
use crate::vectors::qdrant::QdrantClient;
use rusqlite::Connection;
use std::collections::HashMap;

const RRF_K: f32 = 60.0;
const SEMANTIC_SCORE_THRESHOLD: f32 = 0.3;

#[derive(Debug, Clone)]
pub struct HybridSearchResult {
    pub document_id: String,
    pub document_title: String,
    pub filename: String,
    pub chunk_text: String,
    pub rrf_score: f32,
    pub semantic_rank: Option<usize>,
    pub keyword_rank: Option<usize>,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Semantic,
    Keyword,
    Both,
}

impl MatchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchType::Semantic => "semantic",
            MatchType::Keyword => "keyword",
            MatchType::Both => "both",
        }
    }
}

pub async fn hybrid_search(
    conn: &Connection,
    qdrant: &QdrantClient,
    query: &str,
    project_id: &str,
    embedding: Option<Vec<f32>>,
    limit: usize,
) -> Result<Vec<HybridSearchResult>, String> {
    let oversample = limit * 2;

    let semantic_results = if let Some(emb) = embedding {
        if qdrant.is_available().await {
            let collection = crate::vectors::qdrant::get_collection_name(Some(project_id));
            match qdrant.search(&collection, emb, oversample as u64, Some(project_id)).await {
                Ok(results) => results
                    .into_iter()
                    .filter(|r| r.score >= SEMANTIC_SCORE_THRESHOLD)
                    .collect(),
                Err(_) => vec![],
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let keyword_results = fts5_search_documents(conn, query, project_id, oversample)?;

    let merged = merge_with_rrf(semantic_results, keyword_results, limit);
    Ok(merged)
}

fn fts5_search_documents(
    conn: &Connection,
    query: &str,
    project_id: &str,
    limit: usize,
) -> Result<Vec<FtsResult>, String> {
    let query_escaped = escape_fts5_query(query);

    let sql = r#"
        SELECT
            d.id,
            COALESCE(d.title, d.filename) as title,
            d.filename,
            COALESCE(d.content_text, '') as content_text,
            bm25(documents_fts) as score
        FROM documents_fts
        JOIN documents d ON documents_fts.rowid = d.rowid
        WHERE documents_fts MATCH ?1 AND d.project_id = ?2
        ORDER BY score
        LIMIT ?3
    "#;

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let results = stmt
        .query_map(
            rusqlite::params![query_escaped, project_id, limit as i64],
            |row| {
                Ok(FtsResult {
                    document_id: row.get(0)?,
                    title: row.get(1)?,
                    filename: row.get(2)?,
                    content_text: row.get(3)?,
                    score: row.get(4)?,
                })
            },
        )
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

fn escape_fts5_query(query: &str) -> String {
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    words
        .iter()
        .map(|w| {
            if w.contains('"') {
                w.replace('"', "")
            } else {
                (*w).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[derive(Debug)]
struct FtsResult {
    document_id: String,
    title: String,
    filename: String,
    content_text: String,
    #[allow(dead_code)]
    score: f64,
}

fn merge_with_rrf(
    semantic: Vec<crate::vectors::qdrant::SearchResult>,
    keyword: Vec<FtsResult>,
    limit: usize,
) -> Vec<HybridSearchResult> {
    let mut scores: HashMap<String, (f32, Option<usize>, Option<usize>, String, String, String)> =
        HashMap::new();

    for (rank, result) in semantic.iter().enumerate() {
        let rrf_score = 1.0 / (RRF_K + rank as f32 + 1.0);
        let entry = scores
            .entry(result.payload.document_id.clone())
            .or_insert((0.0, None, None, String::new(), String::new(), String::new()));
        entry.0 += rrf_score;
        entry.1 = Some(rank + 1);
        entry.3 = result.payload.chunk_text.clone();
    }

    for (rank, result) in keyword.iter().enumerate() {
        let rrf_score = 1.0 / (RRF_K + rank as f32 + 1.0);
        let entry = scores
            .entry(result.document_id.clone())
            .or_insert((0.0, None, None, String::new(), String::new(), String::new()));
        entry.0 += rrf_score;
        entry.2 = Some(rank + 1);

        if entry.3.is_empty() {
            entry.3 = result.content_text.chars().take(500).collect();
        }
        entry.4 = result.title.clone();
        entry.5 = result.filename.clone();
    }

    for result in &semantic {
        if let Some(entry) = scores.get_mut(&result.payload.document_id) {
            if entry.4.is_empty() {
                entry.4 = result.payload.document_id.clone();
            }
        }
    }

    let mut results: Vec<_> = scores
        .into_iter()
        .map(|(doc_id, (score, sem_rank, kw_rank, chunk, title, filename))| {
            let match_type = match (sem_rank.is_some(), kw_rank.is_some()) {
                (true, true) => MatchType::Both,
                (true, false) => MatchType::Semantic,
                (false, true) => MatchType::Keyword,
                (false, false) => MatchType::Keyword, // Shouldn't happen
            };

            HybridSearchResult {
                document_id: doc_id,
                document_title: title,
                filename,
                chunk_text: chunk,
                rrf_score: score,
                semantic_rank: sem_rank,
                keyword_rank: kw_rank,
                match_type,
            }
        })
        .collect();

    results.sort_by(|a, b| b.rrf_score.partial_cmp(&a.rrf_score).unwrap());
    results.truncate(limit);
    results
}

pub fn search_result_to_model(result: HybridSearchResult) -> SearchResult {
    SearchResult {
        document_id: result.document_id,
        document_title: result.document_title,
        filename: result.filename,
        chunk_text: result.chunk_text.clone(),
        content: result.chunk_text,
        score: result.rrf_score as f64,
        search_type: result.match_type.as_str().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_fts5_query() {
        assert_eq!(escape_fts5_query("hello world"), "hello OR world");
        assert_eq!(escape_fts5_query("test"), "test");
        assert_eq!(escape_fts5_query(""), "");
    }

    #[test]
    fn test_match_type_as_str() {
        assert_eq!(MatchType::Semantic.as_str(), "semantic");
        assert_eq!(MatchType::Keyword.as_str(), "keyword");
        assert_eq!(MatchType::Both.as_str(), "both");
    }

    #[test]
    fn test_rrf_calculation() {
        let score1 = 1.0 / (RRF_K + 0.0 + 1.0);
        let score2 = 1.0 / (RRF_K + 1.0 + 1.0);

        assert!(score1 > score2);
        assert!((score1 - 0.0164).abs() < 0.001);
    }
}
