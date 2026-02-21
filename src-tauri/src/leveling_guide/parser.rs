use crate::error::{command_error, CommandError};
use base64::Engine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

use super::types::{
    GuideAct, GuideData, GuidePage, GuidePosition, LevelingGuideLineDto, LevelingGuidePageDto,
    LevelingGuideSpanDto, LoadedGuide,
};

fn image_path_from_guide_path(guide_path: &str, key: &str) -> Option<PathBuf> {
    let key = key.trim().replace(' ', "_");

    let guide_relative_path = guide_path.strip_prefix("resource:")?;
    let guide_dir = Path::new(guide_relative_path).parent()?;
    Some(guide_dir.join("img").join(format!("{key}.png")))
}

fn load_image_data_uri_uncached(
    app: &AppHandle,
    guide_path: &str,
    normalized_key: &str,
) -> Result<Option<String>, CommandError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e: tauri::Error| command_error("resource_dir_failed", e.to_string()))?;

    let relative_path = match image_path_from_guide_path(guide_path, normalized_key) {
        Some(path) => path,
        None => return Ok(None),
    };

    let absolute_path = resource_dir.join(relative_path);
    let bytes = match std::fs::read(&absolute_path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Some(format!("data:image/png;base64,{encoded}")))
}

fn load_image_data_uri_cached(
    app: &AppHandle,
    guide_path: &str,
    icon_cache: &mut HashMap<String, Option<String>>,
    key: &str,
) -> Result<Option<String>, CommandError> {
    let normalized_key = key.trim().replace(' ', "_");
    if let Some(cached) = icon_cache.get(&normalized_key) {
        return Ok(cached.clone());
    }

    let resolved = load_image_data_uri_uncached(app, guide_path, &normalized_key)?;
    icon_cache.insert(normalized_key, resolved.clone());
    Ok(resolved)
}

fn split_area_id_token(token: &str) -> Option<(&str, &str)> {
    if !token.starts_with("areaid") {
        return None;
    }

    let after_prefix = &token[6..];
    if after_prefix.is_empty() {
        return None;
    }

    let split_index = after_prefix
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(after_prefix.len());

    let (id, suffix) = after_prefix.split_at(split_index);
    Some((id, suffix))
}

fn replace_areaid_tokens_with_area_names(
    line: &str,
    area_name_by_id: &HashMap<String, String>,
) -> String {
    line.split_whitespace()
        .map(|token| {
            let Some((id, suffix)) = split_area_id_token(token) else {
                return token.to_string();
            };

            let Some(name) = area_name_by_id.get(id) else {
                return token.to_string();
            };

            format!("{name}{suffix}")
        })
        .collect::<Vec<String>>()
        .join(" ")
        .trim()
        .to_string()
}

fn strip_comment_and_resolve_area_name(
    line: &str,
    area_name_by_id: &HashMap<String, String>,
) -> String {
    let left = line.split(";;").next().unwrap_or_default().trim();
    replace_areaid_tokens_with_area_names(left, area_name_by_id)
}

fn strip_format_tags(mut line: String) -> String {
    while let Some(start) = line.find("(color:") {
        let end = match line[start..].find(')') {
            Some(offset) => start + offset + 1,
            None => break,
        };
        line.replace_range(start..end, "");
    }

    line.replace("  ", " ").trim().to_string()
}

fn parse_line_is_hint(line: &str) -> (bool, &str) {
    if let Some(rest) = line.strip_prefix("(hint)__") {
        return (true, rest.trim());
    }
    if let Some(rest) = line.strip_prefix("(hint)_") {
        return (true, rest.trim());
    }
    (false, line)
}

fn render_spans(
    app: &AppHandle,
    guide_path: &str,
    icon_cache: &mut HashMap<String, Option<String>>,
    line: &str,
) -> Result<Vec<LevelingGuideSpanDto>, CommandError> {
    let mut spans: Vec<LevelingGuideSpanDto> = Vec::new();
    let mut remaining = line;

    while let Some(start) = remaining.find("(img:") {
        let (before, after_start) = remaining.split_at(start);
        if !before.trim().is_empty() {
            spans.push(LevelingGuideSpanDto::Text {
                text: before.to_string(),
            });
        }

        let after_start = &after_start[5..];
        let end = match after_start.find(')') {
            Some(index) => index,
            None => {
                spans.push(LevelingGuideSpanDto::Text {
                    text: after_start.to_string(),
                });
                return Ok(spans);
            }
        };

        let key = after_start[..end].trim();
        if let Some(data_uri) = load_image_data_uri_cached(app, guide_path, icon_cache, key)? {
            spans.push(LevelingGuideSpanDto::Image {
                key: key.replace(' ', "_"),
                data_uri,
            });
        }

        remaining = after_start[end + 1..].as_ref();
    }

    if !remaining.trim().is_empty() {
        spans.push(LevelingGuideSpanDto::Text {
            text: remaining.to_string(),
        });
    }

    Ok(spans)
}

fn render_line(
    app: &AppHandle,
    guide_path: &str,
    icon_cache: &mut HashMap<String, Option<String>>,
    area_name_by_id: &HashMap<String, String>,
    line: &str,
) -> Result<Option<LevelingGuideLineDto>, CommandError> {
    if line.trim().is_empty() {
        return Ok(None);
    }

    if line.trim_start().starts_with("quest-check:") {
        return Ok(None);
    }

    let stripped = strip_comment_and_resolve_area_name(line, area_name_by_id);
    let stripped = strip_format_tags(stripped);
    let (is_hint, stripped) = parse_line_is_hint(&stripped);

    let spans = render_spans(app, guide_path, icon_cache, stripped)?;
    if spans.is_empty() {
        return Ok(None);
    }

    Ok(Some(LevelingGuideLineDto { is_hint, spans }))
}

pub(crate) fn parse_guide_json(value: serde_json::Value) -> Result<GuideData, CommandError> {
    let acts = value
        .as_array()
        .ok_or_else(|| command_error("guide_invalid_format", "Expected top-level guide array"))?;

    acts
        .iter()
        .map(|act_value| {
            let pages = act_value.as_array().ok_or_else(|| {
                command_error("guide_invalid_format", "Expected act to be an array")
            })?;

            pages
                .iter()
                .map(|page_value| {
                    if let Some(lines) = page_value.as_array() {
                        let parsed_lines = lines
                            .iter()
                            .map(|line| {
                                line.as_str().map(str::to_string).ok_or_else(|| {
                                    command_error(
                                        "guide_invalid_format",
                                        "Expected page line to be a string",
                                    )
                                })
                            })
                            .collect::<Result<Vec<String>, CommandError>>()?;

                        return Ok(GuidePage { lines: parsed_lines });
                    }

                    if let Some(object) = page_value.as_object() {
                        if let Some(lines_value) = object.get("lines") {
                            let lines = lines_value.as_array().ok_or_else(|| {
                                command_error(
                                    "guide_invalid_format",
                                    "Expected conditional page lines to be an array",
                                )
                            })?;

                            let parsed_lines = lines
                                .iter()
                                .map(|line| {
                                    line.as_str().map(str::to_string).ok_or_else(|| {
                                        command_error(
                                            "guide_invalid_format",
                                            "Expected page line to be a string",
                                        )
                                    })
                                })
                                .collect::<Result<Vec<String>, CommandError>>()?;

                            return Ok(GuidePage { lines: parsed_lines });
                        }
                    }

                    Err(command_error(
                        "guide_invalid_format",
                        "Expected page to be an array of strings or object with lines",
                    ))
                })
                .collect::<Result<Vec<GuidePage>, CommandError>>()
        })
        .collect::<Result<Vec<GuideAct>, CommandError>>()
}

pub(crate) fn clamp_position(guide: &GuideData, position: GuidePosition) -> GuidePosition {
    if guide.is_empty() {
        return GuidePosition::start();
    }

    let act_index = position.act_index.min(guide.len().saturating_sub(1));
    let act = &guide[act_index];
    if act.is_empty() {
        return GuidePosition {
            act_index,
            page_index: 0,
        };
    }

    let page_index = position.page_index.min(act.len().saturating_sub(1));

    GuidePosition {
        act_index,
        page_index,
    }
}

pub(crate) fn current_page_dto(
    app: &AppHandle,
    loaded: &mut LoadedGuide,
) -> Result<LevelingGuidePageDto, CommandError> {
    let act = loaded.guide.get(loaded.position.act_index).ok_or_else(|| {
        command_error("guide_position_invalid", "Act index out of bounds")
    })?;

    let page = act.get(loaded.position.page_index).ok_or_else(|| {
        command_error("guide_position_invalid", "Page index out of bounds")
    })?;

    let has_previous = !(loaded.position.act_index == 0 && loaded.position.page_index == 0);

    let has_next = if loaded.position.act_index + 1 < loaded.guide.len() {
        true
    } else {
        loaded.position.page_index + 1 < act.len()
    };

    let lines = page
        .lines
        .iter()
        .map(|line| {
            render_line(
                app,
                &loaded.guide_path,
                &mut loaded.icon_cache,
                &loaded.area_name_by_id,
                line,
            )
        })
        .collect::<Result<Vec<Option<LevelingGuideLineDto>>, CommandError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<LevelingGuideLineDto>>();

    Ok(LevelingGuidePageDto {
        guide_path: loaded.guide_path.clone(),
        position: loaded.position,
        act_count: loaded.guide.len(),
        page_count_in_act: act.len(),
        lines,
        has_previous,
        has_next,
    })
}
