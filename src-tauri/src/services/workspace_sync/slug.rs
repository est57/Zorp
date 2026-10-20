use std::collections::HashSet;

use uuid::Uuid;

const MAX_LEN: usize = 60;

/// Lowercase ASCII name safe on every file system, for example `List users` -> `list-users`.
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
        if out.len() >= MAX_LEN {
            break;
        }
    }
    let out = out.trim_end_matches('-');
    match out {
        "" => "item".into(),
        // Names Windows reserves for devices.
        "con" | "prn" | "aux" | "nul" => format!("{out}-item"),
        _ if (out.starts_with("com") || out.starts_with("lpt"))
            && out.len() == 4
            && out.as_bytes()[3].is_ascii_digit() =>
        {
            format!("{out}-item")
        }
        _ => out.to_string(),
    }
}

/// Hands out unique names within one directory; a clash gets the start of the id appended.
#[derive(Default)]
pub struct Names(HashSet<String>);

impl Names {
    pub fn claim(&mut self, name: &str, id: Uuid) -> String {
        let base = slug(name);
        let mut candidate = base.clone();
        if !self.0.insert(candidate.clone()) {
            candidate = format!("{base}-{}", &id.simple().to_string()[24..]);
            let mut n = 2;
            while !self.0.insert(candidate.clone()) {
                candidate = format!("{base}-{}-{n}", &id.simple().to_string()[24..]);
                n += 1;
            }
        }
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_safe_and_unique() {
        assert_eq!(slug("List users / v2"), "list-users-v2");
        assert_eq!(slug("../.."), "item");
        assert_eq!(slug("CON"), "con-item");
        assert_eq!(slug("Ünïcode Ä"), "n-code");
        let mut names = Names::default();
        let id = Uuid::now_v7();
        assert_eq!(names.claim("Users", id), "users");
        assert!(names.claim("users", id).starts_with("users-"));
    }
}
