//! Autocomplete for markdown.
//! This style of autocomplete will offer a sort of shortcut to snippets,
//! allowing for autocomplete of `delta` to `δ` for instance.

use crate::Buffer;
use crate::autocomplete::{AutoComplete, UpdateRequest};

pub struct Markdown {}
pub const MARKDOWN: Markdown = Markdown {};

fn isnt_token_char(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && c != '-' && c != '\''
}

impl AutoComplete for Markdown {
    fn get_candidates(&self, buf: &Buffer) -> (Vec<String>, usize) {
        let mut query = String::new();
        let line: Vec<char> = buf.contents[buf.cursor_pos.line].chars().collect();
        let mut idx = buf.cursor_pos.idx;
        while idx > 0 {
            idx -= 1;
            if !isnt_token_char(line[idx]) {
                query.push(line[idx])
            } else {
                break;
            }
        }
        query = query.chars().rev().collect();
        (
            Vec::from([(match query.as_str() {
                "degree" => "°",
                "alpha" => "α",
                "beta" => "β",
                "gamma" => "γ",
                "delta" => "δ",
                "epsilon" => "ε",
                "zeta" => "ζ",
                "eta" => "η",
                "theta" => "θ",
                "iota" => "ι",
                "kappa" => "κ",
                "lambda" => "λ",
                "mu" => "μ",
                "nu" => "ν",
                "xi" => "ξ",
                "omicron" => "ο",
                "pi" => "π",
                "rho" => "ρ",
                "sigma" => "σ",
                "tau" => "τ",
                "upsilon" => "υ",
                "phi" => "φ",
                "chi" => "χ",
                "psi" => "ψ",
                "omega" => "ω",
                "Alpha" => "Α",
                "Beta" => "Β",
                "Gamma" => "Γ",
                "Delta" => "Δ",
                "Epsilon" => "Ε",
                "Zeta" => "Ζ",
                "Eta" => "Η",
                "Theta" => "Θ",
                "Iota" => "Ι",
                "Kappa" => "Κ",
                "Lambda" => "Λ",
                "Mu" => "Μ",
                "Nu" => "Ν",
                "Xi" => "Ξ",
                "Omicron" => "Ο",
                "Pi" => "Π",
                "Rho" => "Ρ",
                "Sigma" => "Σ",
                "Tau" => "Τ",
                "Upsilon" => "Υ",
                "Phi" => "Φ",
                "Chi" => "Χ",
                "Psi" => "Ψ",
                "Omega" => "Ω",
                _ => "",
            })
            .to_string()]),
            query.chars().count(),
        )
    }
    fn add_tokens(&mut self, _request: UpdateRequest) {}
    fn is_kind(&self, path: &str) -> bool {
        path.ends_with(".md")
    }
    fn display_str(&self) -> &str {
        "Markdown"
    }
}
