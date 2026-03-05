use crate::error::{command_error, CommandError};
use base64::Engine;
use flate2::read::ZlibDecoder;
use std::collections::HashSet;
use std::io::Read;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PobImportData {
    pub(crate) class: String,
    pub(crate) ascend_class: Option<String>,
    pub(crate) gem_names: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AscendancyClassEntry {
    pub(crate) base_class: String,
    pub(crate) ascendancy_class: String,
}

pub(crate) fn ascendancy_class_list() -> Vec<AscendancyClassEntry> {
    ASCENDANCY_TO_BASE
        .iter()
        .map(|(ascendancy, base)| AscendancyClassEntry {
            base_class: base.to_string(),
            ascendancy_class: ascendancy.to_string(),
        })
        .collect()
}

const ASCENDANCY_TO_BASE: &[(&str, &str)] = &[
    ("reliquarian", "scion"),
    ("ascendant", "scion"),
    ("juggernaut", "marauder"),
    ("berserker", "marauder"),
    ("chieftain", "marauder"),
    ("warden", "ranger"),
    ("deadeye", "ranger"),
    ("pathfinder", "ranger"),
    ("occultist", "witch"),
    ("elementalist", "witch"),
    ("necromancer", "witch"),
    ("slayer", "duelist"),
    ("gladiator", "duelist"),
    ("champion", "duelist"),
    ("inquisitor", "templar"),
    ("hierophant", "templar"),
    ("guardian", "templar"),
    ("assassin", "shadow"),
    ("trickster", "shadow"),
    ("saboteur", "shadow"),
];

fn resolve_base_class(class_name: &str) -> String {
    let lower = class_name.to_ascii_lowercase();
    ASCENDANCY_TO_BASE
        .iter()
        .find(|(asc, _)| *asc == lower)
        .map(|(_, base)| base.to_string())
        .unwrap_or(lower)
}

const STARTER_GEMS: &[(&str, &[&str])] = &[
    ("witch", &["fireball", "arcane surge"]),
    ("shadow", &["viper strike", "chance to poison"]),
    ("ranger", &["burning arrow", "momentum"]),
    ("duelist", &["double strike", "chance to bleed"]),
    ("marauder", &["heavy strike", "ruthless"]),
    ("templar", &["glacial hammer", "elemental proliferation"]),
    ("scion", &["spectral throw", "prismatic burst"]),
];

const EXCLUDED_GEMS: &[&str] = &["empower", "enhance", "enlighten"];

fn starter_gems_for_class(class: &str) -> HashSet<String> {
    let lower = class.to_ascii_lowercase();
    STARTER_GEMS
        .iter()
        .find(|(c, _)| *c == lower)
        .map(|(_, gems)| gems.iter().map(|g| g.to_string()).collect())
        .unwrap_or_default()
}

fn is_excluded_gem(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    EXCLUDED_GEMS.iter().any(|e| lower.contains(e))
}

pub(crate) fn parse_pob_export(pob_code: &str) -> Result<PobImportData, CommandError> {
    let xml = decode_pob_to_xml(pob_code)?;
    extract_pob_data(&xml)
}

fn decode_pob_to_xml(pob_code: &str) -> Result<String, CommandError> {
    let trimmed = pob_code.trim().trim_end_matches('=');

    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(trimmed)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(trimmed))
        .map_err(|e| command_error("pob_decode_failed", format!("Base64 decode failed: {e}")))?;

    let mut decoder = ZlibDecoder::new(&decoded[..]);
    let mut xml = String::new();
    decoder.read_to_string(&mut xml).map_err(|e| {
        command_error(
            "pob_decompress_failed",
            format!("Zlib decompress failed: {e}"),
        )
    })?;

    Ok(xml)
}

fn extract_pob_data(xml: &str) -> Result<PobImportData, CommandError> {
    // TODO: temporary debug dump – remove after inspection
    let _ = std::fs::write("/home/czifumasa/Workspace/poe-tauri/pob.xml", xml);

    let xml_lower = xml.to_ascii_lowercase();

    if !xml_lower.contains("<pathofbuilding>") || !xml_lower.contains("</pathofbuilding>") {
        return Err(command_error(
            "pob_invalid_format",
            "Not a valid Path of Building export",
        ));
    }

    let raw_class = extract_attribute(&xml_lower, "classname").unwrap_or_default();
    let class = resolve_base_class(&raw_class);

    let ascend_class =
        extract_attribute(&xml_lower, "ascendclassname").filter(|v| !v.is_empty() && v != "none");

    let starters = starter_gems_for_class(&class);
    let mut gem_names: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let xml_replaced = xml_lower
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&apos;", "'");

    for gem_tag in find_gem_tags(&xml_replaced) {
        let Some(raw_name) = extract_attribute(&gem_tag, "namespec") else {
            continue;
        };

        if raw_name.is_empty() {
            continue;
        }

        let is_support = gem_tag.contains("/supportgem");

        let mut name = raw_name.clone();
        name = name.replace("vaal ", "");
        name = name.replace("awakened ", "");

        if let Some(colon_pos) = name.find(':') {
            name = name[..colon_pos].to_string();
        }

        if name.is_empty() {
            continue;
        }

        if is_excluded_gem(&name) {
            continue;
        }

        if starters.contains(&name) {
            continue;
        }

        let normalized = if is_support {
            if !name.contains("support") {
                format!("{name} support")
            } else {
                name
            }
        } else {
            name
        };

        if !seen.contains(&normalized) {
            seen.insert(normalized.clone());
            gem_names.push(normalized);
        }
    }

    Ok(PobImportData {
        class,
        ascend_class,
        gem_names,
    })
}

fn extract_attribute(text: &str, attr_name: &str) -> Option<String> {
    let pattern = format!(" {attr_name}=\"");
    let start = text.find(&pattern)? + pattern.len();
    let rest = &text[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn find_gem_tags(xml: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let pattern = "<gem ";
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find(pattern) {
        let abs_start = search_from + start;
        let rest = &xml[abs_start..];

        let end = if let Some(self_close) = rest.find("/>") {
            if let Some(tag_close) = rest.find('>') {
                self_close.min(tag_close) + if self_close < tag_close { 2 } else { 1 }
            } else {
                self_close + 2
            }
        } else if let Some(tag_close) = rest.find('>') {
            tag_close + 1
        } else {
            break;
        };

        tags.push(rest[..end].to_string());
        search_from = abs_start + end;
    }

    tags
}
