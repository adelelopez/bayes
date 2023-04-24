use crate::bayes_component::recalculate;
use crate::chance_component::percentize;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseFloatError;
use std::str::FromStr;
use wasm_bindgen::JsCast;
use web_sys::{window, Blob, BlobPropertyBag, HtmlAnchorElement};

#[derive(Serialize, Deserialize)]
pub struct BayesData {
    pub hypotheses: Vec<String>,
    pub prior_odds: Vec<f64>,
    pub posterior_odds: Vec<f64>,
    pub evidence: Vec<String>,
    pub likelihoods: Vec<Vec<f64>>,
}

#[derive(Debug)]
pub enum MarkdownParseError {
    InvalidFormat(String),
    ParseFloat(ParseFloatError),
}

impl From<ParseFloatError> for MarkdownParseError {
    fn from(err: ParseFloatError) -> MarkdownParseError {
        MarkdownParseError::ParseFloat(err)
    }
}

pub fn parse_markdown(content: &str) -> Result<BayesData, MarkdownParseError> {
    let mut hypotheses: Vec<String> = Vec::new();
    let mut prior_odds: Vec<f64> = Vec::new();
    let mut evidence: Vec<String> = Vec::new();
    let mut likelihoods: Vec<Vec<f64>> = Vec::new();

    let mut current_section: &str = "";

    for line in content.lines() {
        if line.starts_with("##") && !line.starts_with("###") {
            current_section = line.trim_start_matches('#').trim();
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let (hypothesis, value) = line
            .split_once(':')
            .ok_or_else(|| MarkdownParseError::InvalidFormat("Invalid format".to_string()))?;
        let hypothesis = hypothesis.trim().to_string();
        let value = value.trim();

        match current_section {
            "Prior" => {
                prior_odds.push(f64::from_str(value)?);
                hypotheses.push(hypothesis);
            }
            "Evidence" => {
                if line.starts_with("###") {
                    evidence.push(hypothesis.trim_start_matches('#').trim().to_string());
                    likelihoods.push(Vec::new());
                } else {
                    let likelihood = 0.01 * f64::from_str(value.trim_end_matches('%'))?;
                    likelihoods.last_mut().unwrap().push(likelihood);
                }
            }
            "Posterior" => {
                // Posterior will be recalculated
            }
            _ => {}
        }

        // TODO: validation
    }

    let posterior_odds = percentize(recalculate(prior_odds.clone(), likelihoods.clone()));

    Ok(BayesData {
        hypotheses,
        prior_odds,
        posterior_odds,
        evidence,
        likelihoods,
    })
}

pub fn export_to_markdown(state: &BayesData) {
    let markdown = format!("{}", state);
    let blob = Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&markdown.into()),
        BlobPropertyBag::new().type_("text/markdown"),
    )
    .unwrap();
    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let filename = state.hypotheses.join(",") + ".bayes.md";
    let document = window().expect("REASON").document().unwrap();

    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();
    a.set_href(&url);
    a.set_download(&filename);
    document.body().unwrap().append_child(&a).unwrap();
    a.click();
    document.body().unwrap().remove_child(&a).unwrap();
    web_sys::Url::revoke_object_url(&url).unwrap();
}

impl fmt::Display for BayesData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n## Prior")?;
        for (idx, hypothesis) in self.hypotheses.iter().enumerate() {
            writeln!(f, "{}: {}", hypothesis, self.prior_odds[idx])?;
        }

        write!(f, "\n## Evidence")?;
        for (ev_idx, likelihood) in self.likelihoods.iter().enumerate() {
            writeln!(f, "\n### {}:", self.evidence[ev_idx])?;
            for (idx, hypothesis) in self.hypotheses.iter().enumerate() {
                writeln!(f, "{}: {}%", hypothesis, 100.0 * likelihood[idx])?;
            }
        }

        writeln!(f, "\n## Posterior")?;
        for (idx, hypothesis) in self.hypotheses.iter().enumerate() {
            writeln!(f, "{}: {}", hypothesis, self.posterior_odds[idx])?;
        }

        Ok(())
    }
}
