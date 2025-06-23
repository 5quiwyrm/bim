//! Default snippets

use crate::snippets::Snippet;

pub struct Html {}
pub const HTML: Html = Html {};
impl Snippet for Html {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".html")
    }
    fn query(&self, query: &str) -> Vec<String> {
        match query.trim() {
            "init" => {
                vec![
                    "<!DOCTYPE html>".to_string(),
                    "<html lang=\"en\">".to_string(),
                    "  <head>".to_string(),
                    "    <meta charset=\"utf-8\">".to_string(),
                    "    <title>title</title>".to_string(),
                    "  </head>".to_string(),
                    "  <body>".to_string(),
                    "    <!-- page content -->".to_string(),
                    "  </body>".to_string(),
                    "</html>".to_string(),
                ]
            }
            q => vec![format!("<{}>", q), "    ".to_string(), format!("</{}>", q)],
        }
    }
    fn display_str(&self) -> &'static str {
        "html"
    }
}
