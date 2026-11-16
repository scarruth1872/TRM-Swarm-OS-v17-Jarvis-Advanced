use super::error::ParserError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coding {
    pub system: Option<String>,
    pub code: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeableConcept {
    pub coding: Option<Vec<Coding>>,
    pub text: Option<String>,
}

impl CodeableConcept {
    pub fn display_summary(&self) -> String {
        if let Some(text) = &self.text { return text.clone(); }
        if let Some(codings) = &self.coding {
            if let Some(first) = codings.first() {
                if let Some(disp) = &first.display { return disp.clone(); }
                if let Some(code) = &first.code { return code.clone(); }
            }
        }
        "Unnamed Concept".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub system: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanName {
    pub use_: Option<String>,
    pub family: Option<String>,
    pub given: Option<Vec<String>>,
    pub text: Option<String>,
}

impl HumanName {
    pub fn full_name(&self) -> String {
        if let Some(text) = &self.text { return text.clone(); }
        let mut parts = Vec::new();
        if let Some(given) = &self.given { parts.push(given.join(" ")); }
        if let Some(family) = &self.family { parts.push(family.clone()); }
        if parts.is_empty() { "Unknown Name".to_string() } else { parts.join(" ") }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub system: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub reference: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirPatient {
    pub resource_type: String,
    pub id: Option<String>,
    pub active: Option<bool>,
    pub name: Option<Vec<HumanName>>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
    pub identifier: Option<Vec<Identifier>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationComponent {
    pub code: CodeableConcept,
    pub value_quantity: Option<Quantity>,
    pub value_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirObservation {
    pub resource_type: String,
    pub id: Option<String>,
    pub status: String,
    pub code: CodeableConcept,
    pub subject: Option<Reference>,
    pub effective_date_time: Option<String>,
    pub value_quantity: Option<Quantity>,
    pub component: Option<Vec<ObservationComponent>>,
}

pub struct FhirParser;

impl FhirParser {
    pub fn parse_patient(json_str: &str) -> Result<FhirPatient, ParserError> {
        serde_json::from_str(json_str).map_err(|e| ParserError::JsonError(e.to_string()))
    }

    pub fn parse_observation(json_str: &str) -> Result<FhirObservation, ParserError> {
        serde_json::from_str(json_str).map_err(|e| ParserError::JsonError(e.to_string()))
    }
}
