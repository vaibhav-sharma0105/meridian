use calamine::{open_workbook, Reader, Xlsx, Data};
use std::path::Path;

const MAX_ROWS_PER_SHEET: usize = 10_000;

pub fn parse_xlsx(path: &Path) -> Result<String, String> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| format!("Failed to open XLSX file: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let mut output = String::new();

    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            output.push_str(&format!("## {}\n\n", sheet_name));

            let mut row_count = 0;
            let mut headers: Vec<String> = Vec::new();

            for (idx, row) in range.rows().enumerate() {
                if idx == 0 {
                    headers = row.iter()
                        .map(|cell| cell_to_string(cell))
                        .collect();

                    output.push_str("| ");
                    output.push_str(&headers.join(" | "));
                    output.push_str(" |\n");

                    output.push_str("|");
                    for _ in &headers {
                        output.push_str(" --- |");
                    }
                    output.push('\n');
                } else {
                    if row_count >= MAX_ROWS_PER_SHEET {
                        output.push_str(&format!(
                            "\n*... truncated at {} rows (large file)*\n",
                            MAX_ROWS_PER_SHEET
                        ));
                        break;
                    }

                    output.push_str("| ");
                    let cells: Vec<String> = row.iter()
                        .map(|cell| cell_to_string(cell))
                        .collect();
                    output.push_str(&cells.join(" | "));
                    output.push_str(" |\n");

                    row_count += 1;
                }
            }

            output.push_str("\n");
        }
    }

    if output.is_empty() {
        return Err("No readable sheets found in XLSX file".to_string());
    }

    Ok(output)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.replace('|', "\\|").replace('\n', " "),
        Data::Float(f) => {
            if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{:.4}", f).trim_end_matches('0').trim_end_matches('.').to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Data::DateTime(dt) => format!("{}", dt),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#ERROR({:?})", e),
    }
}

pub fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| format!("Failed to open XLSX file: {}", e))?;

    let sheet_names = workbook.sheet_names().to_owned();
    let mut text_parts: Vec<String> = Vec::new();

    for sheet_name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            text_parts.push(format!("Sheet: {}", sheet_name));

            for (idx, row) in range.rows().enumerate() {
                if idx >= MAX_ROWS_PER_SHEET {
                    break;
                }

                let row_text: Vec<String> = row.iter()
                    .filter_map(|cell| {
                        let s = cell_to_string(cell);
                        if s.is_empty() { None } else { Some(s) }
                    })
                    .collect();

                if !row_text.is_empty() {
                    text_parts.push(row_text.join(", "));
                }
            }
        }
    }

    Ok(text_parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_string() {
        assert_eq!(cell_to_string(&Data::Empty), "");
        assert_eq!(cell_to_string(&Data::String("hello".to_string())), "hello");
        assert_eq!(cell_to_string(&Data::Int(42)), "42");
        assert_eq!(cell_to_string(&Data::Float(3.14)), "3.14");
        assert_eq!(cell_to_string(&Data::Float(100.0)), "100");
        assert_eq!(cell_to_string(&Data::Bool(true)), "true");
    }

    #[test]
    fn test_pipe_escape() {
        assert_eq!(
            cell_to_string(&Data::String("hello|world".to_string())),
            "hello\\|world"
        );
    }
}
