use crate::schema::Module;
use crate::errors::McpcError;
use crate::templates;
use std::collections::HashMap;

pub type ModuleOutput = HashMap<String, String>;

pub fn generate_module(module: &Module) -> Result<ModuleOutput, McpcError> {
    let mut output = HashMap::new();

    let main_rs_path = format!("{}/src/main.rs", module.name);
    let main_rs_content = templates::render_main_rs(module)?;
    output.insert(main_rs_path, main_rs_content);

    let lib_rs_path = format!("{}/src/lib.rs", module.name);
    output.insert(lib_rs_path, String::from("pub fn init() {}\n"));

    let cargo_toml_path = format!("{}/Cargo.toml", module.name);
    let cargo_toml_content = templates::render_cargo_toml(module)?;
    output.insert(cargo_toml_path, cargo_toml_content);

    let dockerfile_path = format!("{}/Dockerfile", module.name);
    let dockerfile_content = templates::render_dockerfile(module)?;
    output.insert(dockerfile_path, dockerfile_content);

    let helm_files = templates::render_helm_chart(module)?;
    for (rel_path, content) in helm_files {
        let full_path = format!("{}/{}", module.name, rel_path);
        output.insert(full_path, content);
    }

    Ok(output)
}