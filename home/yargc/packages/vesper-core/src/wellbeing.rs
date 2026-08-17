use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::paths::state_root;
use crate::process::output;

fn today() -> String {
    output("date", &["+%F"]).unwrap_or_else(|_| "unknown-date".to_string())
}

fn normalise_id(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(|ch| ch.to_lowercase())
        .collect()
}

fn load(path: &Path) -> BTreeMap<String, u64> {
    let mut values = BTreeMap::new();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        if let Some((name, seconds)) = line.rsplit_once('\t') {
            if let Ok(seconds) = seconds.parse::<u64>() {
                values.insert(name.to_string(), seconds);
            }
        }
    }
    values
}

pub fn seconds_for(id: &str) -> u64 {
    let target = normalise_id(id.strip_suffix(".desktop").unwrap_or(id));
    if target.is_empty() {
        return 0;
    }
    let path = state_root().join("wellbeing").join(format!("{}.tsv", today()));
    load(&path)
        .into_iter()
        .filter(|(name, _)| {
            let name = normalise_id(name);
            !name.is_empty() && (name.contains(&target) || target.contains(&name))
        })
        .map(|(_, seconds)| seconds)
        .sum()
}
