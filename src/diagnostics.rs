use tracing::{warn, info};

pub struct Diagnostic {
    pub message: String,
    pub file: Option<String>,
    pub help: Option<String>,
}

pub fn report_diagnostic_warning(diag: &Diagnostic) {
    let mut msg = format!("⚠️ Warning: {}", diag.message);
    if let Some(ref file) = diag.file {
        msg.push_str(&format!("\n   --> {}", file));
    }
    if let Some(ref help) = diag.help {
        msg.push_str(&format!("\n   = help: {}", help));
    }
    warn!("{}", msg);
}

pub fn report_warning(msg: &str) {
    warn!("⚠️ Warning: {}", msg);
}

pub fn report_info(msg: &str) {
    info!("ℹ️ Info: {}", msg);
}
