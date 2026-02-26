use crate::error::{command_error, CommandError};
use base64::Engine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

use super::types::{
    GuideAct, GuideData, GuidePage, GuidePosition, LevelingGuideLineDto, LevelingGuidePageDto,
    LevelingGuideHintDto, LevelingGuideSpanDto, LoadedGuide,
};

const BOSS_TARGET_COLOR: &str = "#ff8111";
const AREA_NAME_COLOR_TAG: &str = "fec076";
const HINT_HIGHLIGHT_COLOR: &str = "Aqua";
const QUEST_ITEM_COLOR: &str = "Lime";
const QUEST_REFERENCE_COLOR: &str = "#ffdb1f";

fn image_path_from_guide_path(guide_path: &str, key: &str) -> Option<PathBuf> {
    let key = key.trim().replace(' ', "_");

    let guide_relative_path = guide_path.strip_prefix("resource:")?;
    let guide_dir = Path::new(guide_relative_path).parent()?;
    Some(guide_dir.join("img").join(format!("{key}.png")))
}

fn mime_type_from_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn load_resource_data_uri_uncached(
    app: &AppHandle,
    relative_path: &Path,
) -> Result<Option<String>, CommandError> {
    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e: tauri::Error| command_error("resource_dir_failed", e.to_string()))?;

    let absolute_path = resource_dir.join(relative_path);
    let bytes = match std::fs::read(&absolute_path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };

    let mime_type = mime_type_from_path(relative_path);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(Some(format!("data:{mime_type};base64,{encoded}")))
}

fn load_hint_data_uri_cached(
    app: &AppHandle,
    hint_image_path_by_key: &HashMap<String, PathBuf>,
    hint_image_cache: &mut HashMap<String, Option<String>>,
    key: &str,
) -> Result<Option<String>, CommandError> {
    let normalized_key = key.trim().to_ascii_lowercase();
    if let Some(cached) = hint_image_cache.get(&normalized_key) {
        return Ok(cached.clone());
    }

    let Some(relative_path) = hint_image_path_by_key.get(&normalized_key) else {
        hint_image_cache.insert(normalized_key, None);
        return Ok(None);
    };

    let resolved = load_resource_data_uri_uncached(app, relative_path)?;
    hint_image_cache.insert(normalized_key, resolved.clone());
    Ok(resolved)
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

            let mut words = name
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>();

            if words.is_empty() {
                return token.to_string();
            }

            if !suffix.is_empty() {
                if let Some(last) = words.last_mut() {
                    last.push_str(suffix);
                }
            }

            words
                .into_iter()
                .map(|word| format!("(color:{AREA_NAME_COLOR_TAG}){word}"))
                .collect::<Vec<String>>()
                .join(" ")
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
    line = line.trim().to_string();
    line
}

#[derive(Debug, Clone, Copy)]
struct BossHighlightState {
    previous_was_kill: bool,
}

fn split_segment_preserving_whitespace(segment: &str) -> Vec<(bool, String)> {
    let mut pieces: Vec<(bool, String)> = Vec::new();

    let mut buffer = String::new();
    let mut buffer_is_whitespace: Option<bool> = None;

    for ch in segment.chars() {
        let is_whitespace = ch.is_whitespace();

        match buffer_is_whitespace {
            None => {
                buffer_is_whitespace = Some(is_whitespace);
                buffer.push(ch);
            }
            Some(current_is_whitespace) if current_is_whitespace == is_whitespace => {
                buffer.push(ch);
            }
            Some(current_is_whitespace) => {
                pieces.push((current_is_whitespace, std::mem::take(&mut buffer)));
                buffer_is_whitespace = Some(is_whitespace);
                buffer.push(ch);
            }
        }
    }

    if !buffer.is_empty() {
        pieces.push((buffer_is_whitespace.unwrap_or(false), buffer));
    }

    pieces
}

fn parse_inline_color_tag(mut token: String) -> (Option<String>, String) {
    let mut last_color: Option<String> = None;
    while let Some(start) = token.find("(color:") {
        let after_start = &token[start + 7..];
        let Some(end_offset) = after_start.find(')') else {
            break;
        };
        let color = after_start[..end_offset].trim().to_string();
        last_color = Some(color);

        let end = start + 7 + end_offset + 1;
        token.replace_range(start..end, "");
    }
    (last_color, token)
}

fn parse_inline_hint_tag(mut token: String) -> (Option<String>, String) {
    let mut last_hint: Option<String> = None;
    while let Some(start) = token.find("(hint:") {
        let after_start = &token[start + 6..];
        let Some(end_offset) = after_start.find(')') else {
            break;
        };

        let hint_key = after_start[..end_offset].trim().to_ascii_lowercase();
        last_hint = Some(hint_key);

        let end = start + 6 + end_offset + 1;
        token.replace_range(start..end, "");
    }
    (last_hint, token)
}

fn parse_inline_quest_tag(mut token: String) -> (Option<String>, String) {
    let mut last_quest_item: Option<String> = None;
    while let Some(start) = token.find("(quest:") {
        let after_start = &token[start + 7..];
        let Some(end_offset) = after_start.find(')') else {
            break;
        };

        let quest_item_raw = after_start[..end_offset].trim().to_string();
        let quest_item_display = quest_item_raw.replace('_', " ");
        last_quest_item = Some(quest_item_display.clone());

        let end = start + 7 + end_offset + 1;
        token.replace_range(start..end, &quest_item_display);
    }
    (last_quest_item, token)
}

fn apply_quest_reference_formatting(mut token: String) -> (bool, String) {
    if !(token.contains('<') || token.contains('>')) {
        return (false, token);
    }

    token = token.replace('<', "").replace('>', "");
    token = token.replace('_', " ");

    if token.chars().last().is_some_and(|ch: char| ch.is_ascii_digit()) {
        token.pop();
    }

    (true, token)
}

fn css_color_from_tag(color: &str) -> String {
    let trimmed = color.trim();
    if trimmed.starts_with('#') {
        return trimmed.to_string();
    }

    let is_hex = !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|c: char| c.is_ascii_hexdigit());

    if is_hex && (trimmed.len() == 6 || trimmed.len() == 3) {
        return format!("#{}", trimmed.to_ascii_lowercase());
    }

    trimmed.to_string()
}

fn normalize_token_for_comparison(token: &str) -> String {
    token
        .trim_matches(|c: char| c == ',' || c == '.' || c == ':' || c == ')')
        .to_ascii_lowercase()
}

fn hint_phrase_from_key(key: &str) -> String {
    key.replace('_', " ")
}

fn inject_hint_tags(segment: &str, hint_keys: &[String]) -> String {
    if hint_keys.is_empty() {
        return segment.to_string();
    }

    let lower = segment.to_ascii_lowercase();

    let mut taken: Vec<(usize, usize)> = Vec::new();
    let mut matches: Vec<(usize, usize, String)> = Vec::new();

    for key in hint_keys {
        let phrase = hint_phrase_from_key(key);
        if phrase.is_empty() {
            continue;
        }

        let phrase_lower = phrase.to_ascii_lowercase();
        for (start, _) in lower.match_indices(&phrase_lower) {
            let end = start + phrase_lower.len();
            let overlaps_existing = taken
                .iter()
                .any(|(taken_start, taken_end)| start < *taken_end && end > *taken_start);

            if overlaps_existing {
                continue;
            }

            taken.push((start, end));
            matches.push((start, end, key.clone()));
        }
    }

    if matches.is_empty() {
        return segment.to_string();
    }

    matches.sort_by(|(a_start, _, _), (b_start, _, _)| a_start.cmp(b_start));

    let mut out = String::new();
    let mut cursor: usize = 0;

    for (start, end, key) in matches {
        if start < cursor {
            continue;
        }

        out.push_str(&segment[cursor..start]);
        let matched = &segment[start..end];

        let replaced = matched
            .split_whitespace()
            .map(|word| format!("(hint:{key}){word}"))
            .collect::<Vec<String>>()
            .join(" ");

        out.push_str(&replaced);
        cursor = end;
    }

    out.push_str(&segment[cursor..]);
    out
}

fn render_text_segment_with_boss_highlight(
    app: &AppHandle,
    state: &mut BossHighlightState,
    hint_image_path_by_key: &HashMap<String, PathBuf>,
    hint_image_cache: &mut HashMap<String, Option<String>>,
    segment: &str,
) -> Result<Vec<LevelingGuideSpanDto>, CommandError> {
    let pieces = split_segment_preserving_whitespace(segment);
    let mut spans: Vec<LevelingGuideSpanDto> = Vec::new();

    let mut buffered_text = String::new();
    let mut buffered_color: Option<String> = None;
    let mut buffered_hint_key: Option<String> = None;
    let mut buffered_hint_data_uri: Option<String> = None;

    let flush = |spans: &mut Vec<LevelingGuideSpanDto>,
                 buffered_text: &mut String,
                 buffered_color: &mut Option<String>,
                 buffered_hint_key: &mut Option<String>,
                 buffered_hint_data_uri: &mut Option<String>| {
        if buffered_text.is_empty() {
            return;
        }

        spans.push(LevelingGuideSpanDto::Text {
            text: std::mem::take(buffered_text),
            color: buffered_color.take(),
            hint: buffered_hint_key
                .take()
                .zip(buffered_hint_data_uri.take())
                .map(|(key, data_uri)| LevelingGuideHintDto { key, data_uri }),
        });
    };

    for piece_index in 0..pieces.len() {
        let (is_whitespace, raw_piece) = &pieces[piece_index];
        if raw_piece.is_empty() {
            continue;
        }

        if *is_whitespace {
            if let Some(active_hint_key) = buffered_hint_key.as_deref() {
                let mut next_hint_key: Option<String> = None;
                for (next_is_whitespace, next_piece) in pieces.iter().skip(piece_index + 1) {
                    if next_piece.is_empty() {
                        continue;
                    }
                    if *next_is_whitespace {
                        continue;
                    }
                    let (candidate_hint_key, _) = parse_inline_hint_tag(next_piece.clone());
                    next_hint_key = candidate_hint_key;
                    break;
                }

                if next_hint_key.as_deref() == Some(active_hint_key) {
                    buffered_text.push_str(raw_piece);
                    continue;
                }

                flush(
                    &mut spans,
                    &mut buffered_text,
                    &mut buffered_color,
                    &mut buffered_hint_key,
                    &mut buffered_hint_data_uri,
                );
            }
            if buffered_color.is_some() {
                flush(
                    &mut spans,
                    &mut buffered_text,
                    &mut buffered_color,
                    &mut buffered_hint_key,
                    &mut buffered_hint_data_uri,
                );
            }
            buffered_text.push_str(raw_piece);
            buffered_color = None;
            buffered_hint_key = None;
            buffered_hint_data_uri = None;
            continue;
        }

        let (hint_key, without_hint) = parse_inline_hint_tag(raw_piece.clone());
        let (quest_item, without_quest) = parse_inline_quest_tag(without_hint);
        let (explicit_color, mut text) = parse_inline_color_tag(without_quest);

        let (quest_reference, formatted_text) = apply_quest_reference_formatting(text);
        text = formatted_text;

        let has_arena_prefix = text.contains("arena:");
        if has_arena_prefix {
            text = text.replace("arena:", "");
        }

        if text.is_empty() {
            continue;
        }

        let comparison = normalize_token_for_comparison(&text);

        let excluded_from_kill = comparison == "everything" || comparison == "it";

        let boss_highlight = (state.previous_was_kill && !excluded_from_kill && !comparison.is_empty())
            || (has_arena_prefix && !comparison.is_empty());

        let hint_data_uri = match hint_key.as_deref() {
            Some(key) => load_hint_data_uri_cached(app, hint_image_path_by_key, hint_image_cache, key)?,
            None => None,
        };

        let effective_hint_key = if hint_data_uri.is_some() {
            hint_key.clone()
        } else {
            None
        };

        let color = effective_hint_key
            .as_ref()
            .map(|_| HINT_HIGHLIGHT_COLOR.to_string())
            .or_else(|| explicit_color.as_deref().map(css_color_from_tag))
            .or_else(|| quest_reference.then(|| QUEST_REFERENCE_COLOR.to_string()))
            .or_else(|| quest_item.as_ref().map(|_| QUEST_ITEM_COLOR.to_string()))
            .or_else(|| boss_highlight.then(|| BOSS_TARGET_COLOR.to_string()));

        let should_flush = buffered_color != color
            || buffered_hint_key.as_deref() != effective_hint_key.as_deref();

        if should_flush {
            flush(
                &mut spans,
                &mut buffered_text,
                &mut buffered_color,
                &mut buffered_hint_key,
                &mut buffered_hint_data_uri,
            );
            buffered_color = color;
            buffered_hint_key = effective_hint_key.clone();
            buffered_hint_data_uri = hint_data_uri.clone();
        }
        buffered_text.push_str(&text);

        let is_kill_word = comparison == "kill";
        state.previous_was_kill = is_kill_word;
    }

    flush(
        &mut spans,
        &mut buffered_text,
        &mut buffered_color,
        &mut buffered_hint_key,
        &mut buffered_hint_data_uri,
    );
    Ok(spans)
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
    hint_keys: &[String],
    hint_image_path_by_key: &HashMap<String, PathBuf>,
    hint_image_cache: &mut HashMap<String, Option<String>>,
    line: &str,
) -> Result<Vec<LevelingGuideSpanDto>, CommandError> {
    let mut spans: Vec<LevelingGuideSpanDto> = Vec::new();
    let mut remaining = line;
    let mut highlight_state = BossHighlightState {
        previous_was_kill: false,
    };

    while let Some(start) = remaining.find("(img:") {
        let (before, after_start) = remaining.split_at(start);
        let injected = inject_hint_tags(before, hint_keys);
        spans.extend(render_text_segment_with_boss_highlight(
            app,
            &mut highlight_state,
            hint_image_path_by_key,
            hint_image_cache,
            &injected,
        )?);

        let after_start = &after_start[5..];
        let end = match after_start.find(')') {
            Some(index) => index,
            None => {
                spans.push(LevelingGuideSpanDto::Text {
                    text: after_start.to_string(),
                    color: None,
                    hint: None,
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

    let injected = inject_hint_tags(remaining, hint_keys);
    spans.extend(render_text_segment_with_boss_highlight(
        app,
        &mut highlight_state,
        hint_image_path_by_key,
        hint_image_cache,
        &injected,
    )?);

    Ok(spans)
}

fn render_line(
    app: &AppHandle,
    guide_path: &str,
    icon_cache: &mut HashMap<String, Option<String>>,
    area_name_by_id: &HashMap<String, String>,
    hint_keys: &[String],
    hint_image_path_by_key: &HashMap<String, PathBuf>,
    hint_image_cache: &mut HashMap<String, Option<String>>,
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

    let spans = render_spans(
        app,
        guide_path,
        icon_cache,
        hint_keys,
        hint_image_path_by_key,
        hint_image_cache,
        stripped,
    )?;
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
                &loaded.hint_keys,
                &loaded.hint_image_path_by_key,
                &mut loaded.hint_image_cache,
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
