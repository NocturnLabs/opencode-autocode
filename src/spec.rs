#![allow(dead_code)]
//! App specification schema

use serde::{Deserialize, Serialize};

/// Application specification - technology agnostic

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSpec {
    pub project_name: String,
    pub overview: String,
    pub features: Vec<Feature>,
    pub success_criteria: Vec<String>,

    // Optional sections - filled based on project type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<TechStack>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_endpoints: Option<Vec<ApiEndpoint>>,
}

impl AppSpec {
    /// Create a new empty app spec
    pub fn new(name: &str) -> Self {
        Self {
            project_name: name.to_string(),
            overview: String::new(),
            features: Vec::new(),
            success_criteria: Vec::new(),
            technology: None,
            database: None,
            api_endpoints: None,
        }
    }

    /// Convert to app_spec.md format (XML-like structure used by the prompts)
    pub fn to_spec_text(&self) -> String {
        let mut output = String::new();

        output.push_str("<project_specification>\n");
        output.push_str(&format!(
            "  <project_name>{}</project_name>\n\n",
            self.project_name
        ));

        output.push_str("  <overview>\n");
        output.push_str(&format!("    {}\n", self.overview));
        output.push_str("  </overview>\n\n");

        if let Some(ref tech) = self.technology {
            output.push_str("  <technology_stack>\n");
            if !tech.languages.is_empty() {
                output.push_str(&format!(
                    "    <languages>{}</languages>\n",
                    tech.languages.join(", ")
                ));
            }
            if !tech.frameworks.is_empty() {
                output.push_str(&format!(
                    "    <frameworks>{}</frameworks>\n",
                    tech.frameworks.join(", ")
                ));
            }
            if !tech.tools.is_empty() {
                output.push_str(&format!("    <tools>{}</tools>\n", tech.tools.join(", ")));
            }
            output.push_str("  </technology_stack>\n\n");
        }

        output.push_str("  <core_features>\n");
        for feature in &self.features {
            output.push_str(&format!(
                "    <feature priority=\"{}\">\n",
                feature.priority.as_str()
            ));
            output.push_str(&format!("      <name>{}</name>\n", feature.name));
            output.push_str(&format!(
                "      <description>{}</description>\n",
                feature.description
            ));
            if !feature.sub_features.is_empty() {
                output.push_str("      <sub_features>\n");
                for sub in &feature.sub_features {
                    output.push_str(&format!("        - {}\n", sub));
                }
                output.push_str("      </sub_features>\n");
            }
            output.push_str("    </feature>\n");
        }
        output.push_str("  </core_features>\n\n");

        if let Some(ref db) = self.database {
            output.push_str("  <database>\n");
            output.push_str(&format!("    <type>{}</type>\n", db.db_type));
            output.push_str("    <tables>\n");
            for table in &db.tables {
                output.push_str(&format!("      - {}\n", table));
            }
            output.push_str("    </tables>\n");
            output.push_str("  </database>\n\n");
        }

        if let Some(ref endpoints) = self.api_endpoints {
            output.push_str("  <api_endpoints>\n");
            for ep in endpoints {
                output.push_str("    <endpoint>\n");
                output.push_str(&format!("      <method>{}</method>\n", ep.method));
                output.push_str(&format!("      <path>{}</path>\n", ep.path));
                output.push_str(&format!(
                    "      <description>{}</description>\n",
                    ep.description
                ));
                output.push_str("    </endpoint>\n");
            }
            output.push_str("  </api_endpoints>\n\n");
        }

        output.push_str("  <success_criteria>\n");
        for criterion in &self.success_criteria {
            output.push_str(&format!("    - {}\n", criterion));
        }
        output.push_str("  </success_criteria>\n");

        output.push_str("</project_specification>\n");

        output
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
    pub priority: Priority,
    #[serde(default)]
    pub sub_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: String,
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_to_text() {
        let spec = AppSpec {
            project_name: "Test Project".to_string(),
            overview: "A test project".to_string(),
            features: vec![Feature {
                name: "Feature 1".to_string(),
                description: "A feature".to_string(),
                priority: Priority::High,
                sub_features: vec!["Sub 1".to_string()],
            }],
            success_criteria: vec!["Works correctly".to_string()],
            technology: Some(TechStack {
                languages: vec!["Rust".to_string()],
                frameworks: vec!["Ratatui".to_string()],
                tools: vec![],
            }),
            database: None,
            api_endpoints: None,
        };

        let text = spec.to_spec_text();
        assert!(text.contains("<project_name>Test Project</project_name>"));
        assert!(text.contains("Feature 1"));
    }
}
