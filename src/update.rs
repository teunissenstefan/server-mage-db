use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const RELEASES_URL: &str =
    "https://api.github.com/repos/epenthesis/server-mage-db/releases/latest";

const CHECK_INTERVAL: u64 = 24 * 60 * 60;
const TIMEOUT: Duration = Duration::from_secs(3);
const DISABLE_ENV: &str = "SERVER_NO_UPDATE_CHECK";
const VERSION_ENV: &str = "SERVER_VERSION";

pub fn current() -> String {
    std::env::var(VERSION_ENV).unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string())
}

#[derive(Serialize, Deserialize)]
struct Cache {
    checked_at: u64,
    latest: String,
}

pub fn check(current: &str) -> Option<String> {
    if std::env::var_os(DISABLE_ENV).is_some() {
        return None;
    }
    if cfg!(debug_assertions) && std::env::var_os(VERSION_ENV).is_none() {
        return None;
    }

    let latest = match cached() {
        Some(v) => v,
        None => {
            let v = fetch().ok()?;
            store(&v);
            v
        }
    };

    if latest.is_empty() || !newer(&latest, current) {
        return None;
    }
    Some(latest)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn cache_path() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_CACHE_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => {
            let home = PathBuf::from(std::env::var_os("HOME")?);
            if cfg!(target_os = "macos") {
                home.join("Library").join("Caches")
            } else {
                home.join(".cache")
            }
        }
    };
    Some(base.join("server").join("update.json"))
}

fn cached() -> Option<String> {
    let raw = std::fs::read_to_string(cache_path()?).ok()?;
    let cache: Cache = serde_json::from_str(&raw).ok()?;
    if now().saturating_sub(cache.checked_at) > CHECK_INTERVAL {
        return None;
    }
    Some(cache.latest)
}

fn store(latest: &str) {
    let Some(path) = cache_path() else { return };
    let Some(dir) = path.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let cache = Cache {
        checked_at: now(),
        latest: latest.to_string(),
    };
    if let Ok(data) = serde_json::to_string(&cache) {
        let _ = std::fs::write(path, data);
    }
}

fn fetch() -> Result<String, Box<dyn std::error::Error>> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .user_agent(concat!("server/", env!("CARGO_PKG_VERSION")))
        .build();

    let body = agent
        .get(RELEASES_URL)
        .set("Accept", "application/vnd.github+json")
        .call()?
        .into_string()?;

    let release: Release = serde_json::from_str(&body)?;
    Ok(release.tag_name.trim_start_matches('v').to_string())
}

fn newer(latest: &str, current: &str) -> bool {
    match (parse(latest), parse(current)) {
        (Some(l), Some(c)) => l > c,
        _ => latest != current,
    }
}

fn parse(v: &str) -> Option<[u64; 3]> {
    let v = v.trim_start_matches('v');
    let v = match v.find(['-', '+']) {
        Some(i) => &v[..i],
        None => v,
    };

    let mut parts = v.split('.');
    let out = [
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ];
    if parts.next().is_some() {
        return None;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{newer, parse};

    #[test]
    fn compares_numerically_not_lexically() {
        assert!(newer("0.10.0", "0.9.0"));
        assert!(!newer("0.9.0", "0.10.0"));
        assert!(!newer("1.2.3", "1.2.3"));
        assert!(newer("1.2.4", "1.2.3"));
    }

    #[test]
    fn strips_prefix_and_suffix() {
        assert_eq!(parse("v1.2.3"), Some([1, 2, 3]));
        assert_eq!(parse("1.2.3-rc1"), Some([1, 2, 3]));
        assert_eq!(parse("1.2"), None);
        assert_eq!(parse("1.2.3.4"), None);
    }

    #[test]
    fn unparseable_falls_back_to_inequality() {
        assert!(newer("nightly", "1.2.3"));
        assert!(!newer("nightly", "nightly"));
    }
}
