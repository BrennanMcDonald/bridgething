use std::path::Path;

/// The console is reachable from anywhere on the network, so it asks for a
/// token unless the operator says the network is trusted. The token is minted
/// once and kept, so a restart does not invalidate open tabs.
#[derive(Clone)]
pub struct Auth(Option<String>);

const TOKEN_FILE: &str = "console-token";

impl Auth {
  pub fn open(config_dir: &Path, given: Option<String>, no_auth: bool) -> Self {
    if no_auth {
      return Self(None);
    }
    if let Some(token) = given.filter(|token| !token.trim().is_empty()) {
      return Self(Some(token));
    }
    Self(Some(kept(config_dir)))
  }

  pub fn token(&self) -> Option<&str> {
    self.0.as_deref()
  }

  pub fn allows(&self, presented: Option<&str>) -> bool {
    let Some(wanted) = self.0.as_deref() else {
      return true;
    };
    presented.is_some_and(|held| same(held, wanted))
  }
}

fn kept(config_dir: &Path) -> String {
  let path = config_dir.join(TOKEN_FILE);
  if let Ok(held) = std::fs::read_to_string(&path) {
    let held = held.trim();
    if !held.is_empty() {
      return held.to_owned();
    }
  }
  let fresh = uuid::Uuid::new_v4().simple().to_string();
  if let Err(error) = std::fs::write(&path, &fresh) {
    tracing::warn!(%error, path = %path.display(), "the console token could not be kept; a fresh one is minted each launch");
  }
  fresh
}

fn same(a: &str, b: &str) -> bool {
  a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |seen, (x, y)| seen | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_console_with_no_auth_takes_anyone() {
    let dir = tempfile::tempdir().expect("a scratch directory");
    let auth = Auth::open(dir.path(), None, true);

    assert!(auth.token().is_none());
    assert!(auth.allows(None), "an open console does not ask");
  }

  #[test]
  fn a_token_outlives_the_process_that_minted_it() {
    let dir = tempfile::tempdir().expect("a scratch directory");

    let first = Auth::open(dir.path(), None, false).token().expect("a minted token").to_owned();

    assert_eq!(
      Auth::open(dir.path(), None, false).token(),
      Some(first.as_str()),
      "a restart does not lock an open tab out"
    );
  }

  #[test]
  fn only_the_token_gets_in() {
    let dir = tempfile::tempdir().expect("a scratch directory");
    let auth = Auth::open(dir.path(), Some("swordfish".to_owned()), false);

    assert!(auth.allows(Some("swordfish")));
    assert!(!auth.allows(Some("swordfis")), "a prefix is not the token");
    assert!(!auth.allows(Some("swordfisH")));
    assert!(!auth.allows(None), "no token is not the token");
  }
}
