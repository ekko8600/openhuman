use crate::types::DocumentFormat;
use anyhow::{bail, Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn detect_format(path: &Path) -> Result<DocumentFormat> {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .context("document path has no extension")?;
    DocumentFormat::from_extension(ext)
        .with_context(|| format!("unsupported document extension: {ext}"))
}

pub fn parse_document(path: &Path, format: &DocumentFormat) -> Result<String> {
    match format {
        DocumentFormat::Markdown | DocumentFormat::Text => Ok(fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?),
        DocumentFormat::Html => html_to_text(&fs::read_to_string(path)?),
        DocumentFormat::Pdf => parse_pdf(path),
    }
}

fn parse_pdf(path: &Path) -> Result<String> {
    let text = pdf_extract::extract_text(path)
        .with_context(|| format!("failed to extract PDF text from {}", path.display()))?;
    if text.trim().is_empty() {
        bail!("PDF text extraction produced no text; OCR is not implemented yet")
    }
    Ok(text)
}

fn html_to_text(html: &str) -> Result<String> {
    let without_scripts = Regex::new(r"(?is)<script[^>]*>.*?</script>")?.replace_all(html, " ");
    let without_scripts =
        Regex::new(r"(?is)<style[^>]*>.*?</style>")?.replace_all(&without_scripts, " ");
    let with_breaks =
        Regex::new(r"(?i)</?(p|div|section|article|header|footer|main|br|li|h[1-6])[^>]*>")?
            .replace_all(&without_scripts, "\n");
    let without_tags = Regex::new(r"(?is)<[^>]+>")?.replace_all(&with_breaks, " ");
    let decoded = decode_basic_html_entities(&without_tags);
    let normalized = decoded
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(normalized)
}

fn decode_basic_html_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_uppercase_txt_extension() {
        assert_eq!(
            detect_format(Path::new("paper.TXT")).unwrap(),
            DocumentFormat::Text
        );
    }

    #[test]
    fn strips_html_to_readable_text() {
        let text = html_to_text("<h1>Title</h1><script>x()</script><p>A&amp;B</p>").unwrap();
        assert!(text.contains("Title"));
        assert!(text.contains("A&B"));
        assert!(!text.contains("script"));
    }
}
