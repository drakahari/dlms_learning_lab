//! Development-only stdin probe for fixed fstab parsing comparisons; no commands.
use dlms_learning_lab::storage::{self, Source};
use std::io::{self, Read};
fn main() {
    let mut text = String::new();
    io::stdin()
        .take((storage::MAX_CONFIG_BYTES + 1) as u64)
        .read_to_string(&mut text)
        .unwrap();
    let value = match storage::parse(&text) {
        Ok(entries) => serde_json::json!({"entries":entries.iter().map(|e| serde_json::json!({
            "source":match &e.source {Source::Device(s)=>s.clone(),Source::Uuid(s)=>format!("UUID={s}")},
            "target":e.target,"fstype":e.filesystem.name(),"read_only":e.read_only,"automatic":e.automatic,"freq":e.dump,"passno":e.pass
        })).collect::<Vec<_>>()}),
        Err(error) => {
            serde_json::json!({"error":error.to_string(),"kind":format!("{:?}",error.kind)})
        }
    };
    println!("{value}");
}
