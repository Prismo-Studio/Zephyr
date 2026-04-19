use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use eyre::{bail, Context};
use tauri::{command, AppHandle, Manager};
use tracing::{debug, warn};

use crate::util::cmd::Result;

fn cache_dir(app: &AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("icon_cache")
}

fn cache_file_for(app: &AppHandle, url: &str) -> PathBuf {
    let hash = blake3::hash(url.as_bytes()).to_hex().to_string();
    let ext = url
        .rsplit('?')
        .next_back()
        .unwrap_or(url)
        .rsplit('/')
        .next()
        .and_then(|name| name.rsplit_once('.'))
        .map(|(_, ext)| ext.to_lowercase())
        .filter(|ext| ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "img".to_string());

    cache_dir(app).join(format!("{hash}.{ext}"))
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(format!("zephyr-icon-cache/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build http client")
    })
}

const MAX_BYTES: usize = 8 * 1024 * 1024;

async fn download_into_cache(app: &AppHandle, url: &str, dest: &PathBuf) -> eyre::Result<()> {
    debug!("icon-cache: fetching {url}");
    let response = http_client()
        .get(url)
        .send()
        .await
        .with_context(|| format!("fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("status for {url}"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("read body of {url}"))?;

    if bytes.len() > MAX_BYTES {
        bail!("icon too large ({} bytes) for {url}", bytes.len());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let tmp = dest.with_extension("download.tmp");
    std::fs::write(&tmp, &bytes).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, dest).with_context(|| format!("rename to {}", dest.display()))?;

    Ok(())
}

#[command]
pub async fn get_cached_icon(url: String, app: AppHandle) -> Result<String> {
    if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
        return Ok(url);
    }

    let dest = cache_file_for(&app, &url);

    if dest.exists() {
        if let Ok(meta) = std::fs::metadata(&dest) {
            if meta.len() > 0 {
                return Ok(dest.to_string_lossy().to_string());
            }
        }
    }

    match download_into_cache(&app, &url, &dest).await {
        Ok(()) => Ok(dest.to_string_lossy().to_string()),
        Err(err) => {
            warn!("icon-cache miss for {url}: {err:#}");
            Ok(url)
        }
    }
}

#[command]
pub fn clear_icon_cache(app: AppHandle) -> Result<u64> {
    let dir = cache_dir(&app);
    if !dir.exists() {
        return Ok(0);
    }
    let mut freed: u64 = 0;
    for entry in std::fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if let Ok(meta) = entry.metadata() {
            freed = freed.saturating_add(meta.len());
        }
        let _ = std::fs::remove_file(entry.path());
    }
    Ok(freed)
}

pub fn evict_older_than(app: &AppHandle, max_age: Duration) -> u64 {
    let dir = cache_dir(app);
    if !dir.exists() {
        return 0;
    }
    let cutoff = match SystemTime::now().checked_sub(max_age) {
        Some(t) => t,
        None => return 0,
    };
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut freed: u64 = 0;
    for entry in entries.flatten() {
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if modified < cutoff {
            let size = meta.len();
            if std::fs::remove_file(entry.path()).is_ok() {
                freed = freed.saturating_add(size);
            }
        }
    }
    freed
}
