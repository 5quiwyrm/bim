//! Snippets for Markdown
//! This is specifically intended for chemistry notes, where greek letters and
//! special symbols not available on my keyboard are present.

use crate::snippets::Snippet;

pub struct Markdown {}
pub const MARKDOWN: Markdown = Markdown {};

impl Snippet for Markdown {
    fn is_kind(&self, filepath: &str) -> bool {
        filepath.ends_with(".md")
    }
    fn query(&self, query: &str) -> Vec<String> {
        Vec::from([(match query {
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
        .to_string()])
    }
    fn display_str(&self) -> &'static str {
        "Markdown"
    }
}
