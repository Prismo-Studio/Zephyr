//! Centralised URLs and other tunable constants.
//!
//! Values that talk to external services live here so they can be swapped or
//! reviewed without grepping the codebase. Endpoints that are part of a
//! provider's documented API (Thunderstore, archipelago.gg, GitHub) are kept
//! as plain `&'static str`. Cloudinary and the Zephyr cloud get helper
//! constants because they appear in multiple call sites.

// ── Thunderstore ──────────────────────────────────────────────────────────

pub const THUNDERSTORE_LEGACY_PROFILE_CREATE: &str =
    "https://thunderstore.io/api/experimental/legacyprofile/create/";

pub const THUNDERSTORE_LEGACY_PROFILE_GET: &str =
    "https://thunderstore.io/api/experimental/legacyprofile/get/{key}/";

// ── archipelago.gg ────────────────────────────────────────────────────────

pub const ARCHIPELAGO_GG_BASE: &str = "https://archipelago.gg";
pub const ARCHIPELAGO_GG_UPLOADS: &str = "https://archipelago.gg/uploads";

// ── Cloudinary (Zephyr media bucket) ──────────────────────────────────────

pub const CLOUDINARY_AUTO_UPLOAD: &str =
    "https://api.cloudinary.com/v1_1/djmsz47e5/auto/upload";
pub const CLOUDINARY_IMAGE_UPLOAD: &str =
    "https://api.cloudinary.com/v1_1/djmsz47e5/image/upload";

// ── GitHub (games.json discovery) ─────────────────────────────────────────

pub const GAMES_JSON_COMMITS_URL: &str =
    "https://api.github.com/repos/Prismo-Studio/Zephyr/commits?path=src-tauri/games.json&per_page=1";

pub const GAMES_JSON_RAW_URL: &str =
    "https://raw.githubusercontent.com/Prismo-Studio/Zephyr/refs/heads/dev/src-tauri/games.json";

// ── Zephyr cloud ──────────────────────────────────────────────────────────

pub const ZEPHYR_CLOUD_DEFAULT: &str = "https://api.zephyr.prismo-studios.dev";

// ── Public IP detection fallbacks ─────────────────────────────────────────
//
// Tried in order; the first one to succeed wins. Multiple providers because
// any of them can be down or geo-blocked at any time.
pub const PUBLIC_IP_PROVIDERS: &[&str] = &[
    "https://api.ipify.org",
    "https://ifconfig.me/ip",
    "https://icanhazip.com",
];
