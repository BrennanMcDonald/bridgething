use std::path::{Path, PathBuf};

const INDEX: &str = "index.html";

/// Where the built console lives. An install drops it beside the binary; a
/// checkout has it under `headless/dist` after `bun run build`.
pub fn resolve(explicit: Option<PathBuf>) -> Option<PathBuf> {
  if let Some(root) = explicit {
    return holds_console(&root).then_some(root);
  }

  let exe = std::env::current_exe().ok();
  let beside = exe.as_deref().and_then(Path::parent).map(Path::to_path_buf);

  let candidates = [
    beside.as_ref().map(|dir| dir.join("web")),
    beside.as_ref().map(|dir| dir.join("../share/bridgething-console/web")),
    Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist")),
  ];

  candidates
    .into_iter()
    .flatten()
    .find(|root| holds_console(root))
    .map(|root| root.canonicalize().unwrap_or(root))
}

fn holds_console(root: &Path) -> bool {
  root.join(INDEX).is_file()
}

pub fn index_of(root: &Path) -> PathBuf {
  root.join(INDEX)
}
