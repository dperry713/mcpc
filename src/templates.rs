use crate::schema::Module;
use minijinja::render;
use std::collections::HashMap;

pub fn render_main_rs(module: &Module) -> String {
    let module_type = module.module_type.as_deref().unwrap_or("default");
    
    match module_type {
        "api" => {
            render!(
r#"use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(|| async { "OK" }));
    println!("API '{{ name }}' listening on 0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
"#, name => module.name
            )
        }
        "worker" => {
            render!(
r#"use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Worker '{{ name }}' starting...");
    loop {
        println!("Worker '{{ name }}' heartbeat...");
        sleep(Duration::from_secs(5)).await;
    }
}
"#, name => module.name
            )
        }
        "agent" => {
            render!(
r#"fn main() {
    println!("Agent '{{ name }}' initialized.");
}
"#, name => module.name
            )
        }
        _ => {
            render!(
r#"fn main() {
    println!("Running module: {{ name }}");
}
"#, name => module.name
            )
        }
    }
}

pub fn render_cargo_toml(module: &Module) -> String {
    let module_type = module.module_type.as_deref().unwrap_or("default");
    let mut toml = render!(
r#"[package]
name = "{{ name }}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#, name => module.name
    );

    for dep in &module.dependencies {
        toml.push_str(&render!(r#"{{ dep }} = { path = "../{{ dep }}" }"#, dep => dep));
        toml.push('\n');
    }

    match module_type {
        "api" => {
            toml.push_str("tokio = { version = \"1.0\", features = [\"full\"] }\n");
            toml.push_str("axum = \"0.7\"\n");
        }
        "worker" => {
            toml.push_str("tokio = { version = \"1.0\", features = [\"full\"] }\n");
        }
        _ => {}
    }

    toml
}

pub fn render_dockerfile(module: &Module) -> String {
    render!(
r#"FROM rust:1.77 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/{{ name }} /usr/local/bin/{{ name }}
CMD ["{{ name }}"]
"#, name => module.name
    )
}

pub fn render_helm_chart(module: &Module) -> HashMap<String, String> {
    let mut files = HashMap::new();
    let module_type = module.module_type.as_deref().unwrap_or("default");

    files.insert(
        "charts/Chart.yaml".into(),
        render!(
r#"apiVersion: v2
name: {{ name }}
description: A Helm chart for {{ name }}
type: application
version: 0.1.0
appVersion: "1.0.0"
"#, name => module.name
        )
    );

    files.insert(
        "charts/values.yaml".into(),
        render!(
r#"replicaCount: 1
image:
  repository: {{ name }}
  pullPolicy: IfNotPresent
  tag: "latest"
"#, name => module.name
        )
    );

    files.insert(
        "charts/templates/deployment.yaml".into(),
        render!(
r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ "{{" }} include "{{ name }}.fullname" . {{ "}}" }}
  labels:
    app.kubernetes.io/name: {{ "{{" }} include "{{ name }}.name" . {{ "}}" }}
spec:
  replicas: {{ "{{" }} .Values.replicaCount {{ "}}" }}
  selector:
    matchLabels:
      app.kubernetes.io/name: {{ "{{" }} include "{{ name }}.name" . {{ "}}" }}
  template:
    metadata:
      labels:
        app.kubernetes.io/name: {{ "{{" }} include "{{ name }}.name" . {{ "}}" }}
    spec:
      containers:
        - name: {{ "{{" }} .Chart.Name {{ "}}" }}
          image: "{{ "{{" }} .Values.image.repository {{ "}}" }}:{{ "{{" }} .Values.image.tag {{ "}}" }}"
"#, name => module.name
        )
    );

    if module_type == "api" {
        files.insert(
            "charts/templates/service.yaml".into(),
            render!(
r#"apiVersion: v1
kind: Service
metadata:
  name: {{ "{{" }} include "{{ name }}.fullname" . {{ "}}" }}
spec:
  type: ClusterIP
  ports:
    - port: 3000
      targetPort: 3000
      protocol: TCP
      name: http
  selector:
    app.kubernetes.io/name: {{ "{{" }} include "{{ name }}.name" . {{ "}}" }}
"#, name => module.name
            )
        );
    }

    files
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cargo_toml() {
        let module = Module {
            name: "agent".into(),
            module_type: Some("worker".into()),
            entry: None,
            features: vec![],
            dependencies: vec!["control-plane".into()],
        };
        let toml = render_cargo_toml(&module);
        assert!(toml.contains("name = \"agent\""));
        assert!(toml.contains("control-plane = { path = \"../control-plane\" }"));
        assert!(toml.contains("tokio = "));
    }
}