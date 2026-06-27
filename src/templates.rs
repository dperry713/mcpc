use crate::schema::Module;
use handlebars::{Handlebars, DirectorySourceOptions};
use rust_embed::RustEmbed;
use std::collections::HashMap;
use serde_json::json;
use std::path::Path;
use crate::errors::McpcError;

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct EmbeddedTemplates;

pub struct TemplateEngine<'a> {
    pub registry: Handlebars<'a>,
}

impl<'a> TemplateEngine<'a> {
    pub fn new() -> Result<Self, McpcError> {
        let mut registry = Handlebars::new();
        // 1. Register embedded templates
        for file in EmbeddedTemplates::iter() {
            if let Some(content) = EmbeddedTemplates::get(file.as_ref()) {
                let template_str = std::str::from_utf8(content.data.as_ref())
                    .map_err(|e| McpcError::Build(format!("Invalid UTF-8 in template {}: {}", file, e)))?;
                registry.register_template_string(file.as_ref(), template_str)
                    .map_err(|e| McpcError::Build(format!("Failed to register template {}: {}", file, e)))?;
            }
        }
        
        // 2. Register local overrides
        let local_templates_dir = Path::new(".mcpc/templates");
        if local_templates_dir.exists() && local_templates_dir.is_dir() {
            let mut options = DirectorySourceOptions::default();
            options.tpl_extension = ".hbs".to_string();
            registry.register_templates_directory(local_templates_dir, options)
                .map_err(|e| McpcError::Build(format!("Failed to load local templates: {}", e)))?;
        }

        Ok(Self { registry })
    }

    pub fn render(&self, template_name: &str, data: &serde_json::Value) -> Result<String, McpcError> {
        self.registry.render(template_name, data)
            .map_err(|e| McpcError::Build(format!("Failed to render template {}: {}", template_name, e)))
    }
}

pub fn render_main_rs(module: &Module) -> Result<String, McpcError> {
    let engine = TemplateEngine::new()?;
    let module_type = module.module_type.as_deref().unwrap_or("default");
    let template_name = format!("{}_main.rs.hbs", module_type);
    
    let data = json!({
        "name": module.name,
        "features": module.features,
    });
    
    // Fallback to default if type-specific doesn't exist
    if engine.registry.has_template(&template_name) {
        engine.render(&template_name, &data)
    } else {
        engine.render("default_main.rs.hbs", &data)
    }
}

pub fn render_cargo_toml(module: &Module) -> Result<String, McpcError> {
    let engine = TemplateEngine::new()?;
    let data = json!({
        "name": module.name,
        "dependencies": module.dependencies,
        "module_type": module.module_type.as_deref().unwrap_or("default"),
    });
    engine.render("cargo_toml.hbs", &data)
}

pub fn render_dockerfile(module: &Module) -> Result<String, McpcError> {
    let engine = TemplateEngine::new()?;
    let data = json!({
        "name": module.name,
    });
    engine.render("dockerfile.hbs", &data)
}

pub fn render_helm_chart(module: &Module) -> Result<HashMap<String, String>, McpcError> {
    let mut files = HashMap::new();
    let engine = TemplateEngine::new()?;
    
    let data = json!({
        "name": module.name,
        "module_type": module.module_type.as_deref().unwrap_or("default"),
        "open": "{{",
        "close": "}}"
    });

    files.insert("charts/Chart.yaml".into(), engine.render("helm/Chart.yaml.hbs", &data)?);
    files.insert("charts/values.yaml".into(), engine.render("helm/values.yaml.hbs", &data)?);
    files.insert("charts/templates/deployment.yaml".into(), engine.render("helm/deployment.yaml.hbs", &data)?);

    if module.module_type.as_deref() == Some("api") {
        files.insert("charts/templates/service.yaml".into(), engine.render("helm/service.yaml.hbs", &data)?);
    }

    Ok(files)
}