use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use cookie_store::{CookieDomain, CookieExpiration, CookieStore, RawCookie};
use reqwest::header::HeaderValue;
use url::Url;

use crate::error::{AppError, AppResult};
use crate::models::cookie::CookieInfo;

/// Workspace cookie jar shared by every HTTP client and persisted to SQLite.
#[derive(Default)]
pub struct CookieJar {
    store: RwLock<CookieStore>,
    dirty: AtomicBool,
}

fn domain_of(domain: &CookieDomain) -> Option<(&str, bool)> {
    match domain {
        CookieDomain::HostOnly(d) => Some((d, true)),
        CookieDomain::Suffix(d) => Some((d, false)),
        _ => None,
    }
}

impl CookieJar {
    pub fn from_json(json: &str) -> Self {
        let store = cookie_store::serde::json::load_all(json.as_bytes()).unwrap_or_default();
        Self {
            store: RwLock::new(store),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn to_json(&self) -> AppResult<String> {
        let store = self
            .store
            .read()
            .map_err(|_| AppError::Internal("Cookie jar terkunci".into()))?;
        let mut out = Vec::new();
        cookie_store::serde::json::save_incl_expired_and_nonpersistent(&store, &mut out)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(String::from_utf8_lossy(&out).into_owned())
    }

    /// Swaps in the cookies of another workspace.
    pub fn replace(&self, json: Option<&str>) -> AppResult<()> {
        let store = json
            .and_then(|j| cookie_store::serde::json::load_all(j.as_bytes()).ok())
            .unwrap_or_default();
        *self
            .store
            .write()
            .map_err(|_| AppError::Internal("Cookie jar terkunci".into()))? = store;
        self.dirty.store(false, Ordering::Release);
        Ok(())
    }

    /// Returns true once after cookies changed, so callers know when to persist.
    pub fn take_dirty(&self) -> bool {
        self.dirty.swap(false, Ordering::AcqRel)
    }

    pub fn list(&self) -> Vec<CookieInfo> {
        let Ok(store) = self.store.read() else {
            return Vec::new();
        };
        let mut items: Vec<CookieInfo> = store
            .iter_unexpired()
            .filter_map(|c| {
                let (domain, host_only) = domain_of(&c.domain)?;
                Some(CookieInfo {
                    domain: domain.to_string(),
                    path: c.path.as_ref().to_string(),
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    expires_at: match c.expires {
                        CookieExpiration::AtUtc(at) => Some(at.unix_timestamp() * 1000),
                        CookieExpiration::SessionEnd => None,
                    },
                    secure: c.secure().unwrap_or(false),
                    http_only: c.http_only().unwrap_or(false),
                    host_only,
                })
            })
            .collect();
        items.sort_by(|a, b| (&a.domain, &a.name).cmp(&(&b.domain, &b.name)));
        items
    }

    pub fn remove(&self, domain: &str, path: &str, name: &str) {
        if let Ok(mut store) = self.store.write() {
            store.remove(domain, path, name);
            self.dirty.store(true, Ordering::Release);
        }
    }

    /// Removes all cookies, or only those for `domain`.
    pub fn clear(&self, domain: Option<&str>) {
        let Ok(mut store) = self.store.write() else {
            return;
        };
        match domain {
            None => store.clear(),
            Some(target) => {
                let keys: Vec<(String, String, String)> = store
                    .iter_any()
                    .filter_map(|c| {
                        let (d, _) = domain_of(&c.domain)?;
                        (d == target).then(|| {
                            (
                                d.to_string(),
                                c.path.as_ref().to_string(),
                                c.name().to_string(),
                            )
                        })
                    })
                    .collect();
                for (d, p, n) in keys {
                    store.remove(&d, &p, &n);
                }
            }
        }
        self.dirty.store(true, Ordering::Release);
    }
}

impl reqwest::cookie::CookieStore for CookieJar {
    fn set_cookies(&self, headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let cookies = headers
            .filter_map(|h| h.to_str().ok())
            .filter_map(|s| RawCookie::parse(s.to_string()).ok())
            .collect::<Vec<_>>();
        if cookies.is_empty() {
            return;
        }
        if let Ok(mut store) = self.store.write() {
            store.store_response_cookies(cookies.into_iter(), url);
            self.dirty.store(true, Ordering::Release);
        }
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let store = self.store.read().ok()?;
        let header = store
            .get_request_values(url)
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ");
        if header.is_empty() {
            None
        } else {
            HeaderValue::from_str(&header).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::cookie::CookieStore as _;

    #[test]
    fn stores_matches_and_round_trips_cookies() {
        let jar = CookieJar::default();
        let url = Url::parse("https://api.test/login").unwrap();
        let headers = [
            HeaderValue::from_static("session=abc; Path=/; HttpOnly; Secure"),
            HeaderValue::from_static("theme=dark; Max-Age=3600"),
        ];
        jar.set_cookies(&mut headers.iter(), &url);
        assert!(jar.take_dirty());

        let sent = jar
            .cookies(&Url::parse("https://api.test/users").unwrap())
            .unwrap();
        assert!(sent.to_str().unwrap().contains("session=abc"));
        assert!(
            jar.cookies(&Url::parse("https://other.test/").unwrap())
                .is_none()
        );

        let restored = CookieJar::from_json(&jar.to_json().unwrap());
        assert_eq!(restored.list().len(), 2);
        restored.clear(Some("api.test"));
        assert!(restored.list().is_empty());
    }
}
