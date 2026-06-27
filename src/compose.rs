use crate::schema::Module;

pub fn render_docker_compose<'a>(modules: impl Iterator<Item = &'a Module>) -> String {
    let mut out = String::new();
    out.push_str("version: '3.8'\n");
    out.push_str("services:\n");

    let mut port_offset = 0;

    for module in modules {
        out.push_str(&format!("  {}:\n", module.name));
        out.push_str(&format!("    build: ./{}\n", module.name));
        
        let module_type = module.module_type.as_deref().unwrap_or("default");
        if module_type == "api" {
            let host_port = 3000 + port_offset;
            out.push_str("    ports:\n");
            out.push_str(&format!("      - \"{}:3000\"\n", host_port));
            port_offset += 1;
        }

        if !module.dependencies.is_empty() {
            out.push_str("    depends_on:\n");
            for dep in &module.dependencies {
                out.push_str(&format!("      - {}\n", dep));
            }
        }
    }

    out
}

