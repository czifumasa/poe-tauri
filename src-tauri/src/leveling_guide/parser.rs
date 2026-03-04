use crate::error::{command_error, CommandError};
use crate::persistence::settings::LevelingGuideSettings;
use base64::Engine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri::Manager;

use super::types::{
    GuideAct, GuideCondition, GuideData, GuidePage, GuidePosition, LevelingGuideHintDto,
    LevelingGuideLineDto, LevelingGuidePageDto, LevelingGuideSpanDto, LoadedGuide,
};

const BOSS_TARGET_COLOR: &str = "#ff8111";
const AREA_NAME_COLOR_TAG: &str = "fec076";
const HINT_HIGHLIGHT_COLOR: &str = "Aqua";
const QUEST_ITEM_COLOR: &str = "Lime";
const QUEST_REFERENCE_COLOR: &str = "#ffdb1f";
const ARENA_IMAGE_FOLLOWUP_COLOR: &str = "#cc99ff";
const TRIAL_OR_LAB_COLOR: &str = "#569777";

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

fn extract_last_area_id_from_page(page: &GuidePage) -> Option<String> {
    let mut last_area_id: Option<String> = None;

    for line in &page.lines {
        let trimmed = line.trim();
        if trimmed.starts_with("(hint)_") {
            continue;
        }

        let without_comment = trimmed.split(";;").next().unwrap_or_default();

        for token in without_comment.split_whitespace() {
            if let Some((id, _)) = split_area_id_token(token) {
                last_area_id = Some(id.to_string());
            }
        }
    }

    last_area_id
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
    line = line.replace("(a11)", "(epilogue)");
    line
}

fn strip_level_recommendation_tokens(line: &str) -> String {
    let mut out = String::new();
    let mut cursor: usize = 0;

    while let Some(offset) = line[cursor..].find("(lvl:") {
        let start = cursor + offset;
        out.push_str(&line[cursor..start]);

        if out.ends_with(' ') {
            out.pop();
        }

        let Some(end_offset) = line[start..].find(')') else {
            out.push_str(&line[start..]);
            return out;
        };

        cursor = start + end_offset + 1;
    }

    out.push_str(&line[cursor..]);
    out
}

fn render_level_recommendation_tokens(line: &str) -> String {
    let mut out = String::new();
    let mut cursor: usize = 0;

    while let Some(offset) = line[cursor..].find("(lvl:") {
        let start = cursor + offset;
        out.push_str(&line[cursor..start]);

        let Some(end_offset) = line[start..].find(')') else {
            out.push_str(&line[start..]);
            return out;
        };

        let token_end = start + end_offset;
        let raw_level_range = line[start + 5..token_end].trim();
        if raw_level_range.is_empty() {
            out.push_str(&line[start..=token_end]);
        } else {
            out.push_str("(");
            out.push_str(raw_level_range);
            out.push_str(" lvl)");
        }

        cursor = token_end + 1;
    }

    out.push_str(&line[cursor..]);
    out
}

fn apply_level_recommendation_setting(line: &str, settings: &LevelingGuideSettings) -> String {
    if settings.level_recommendations {
        return render_level_recommendation_tokens(line);
    }
    strip_level_recommendation_tokens(line)
}

fn bandit_key(settings: &LevelingGuideSettings) -> &'static str {
    use crate::persistence::settings::BanditsChoice;

    match settings.bandits_choice {
        BanditsChoice::KillAll => "none",
        BanditsChoice::HelpAlira => "alira",
        BanditsChoice::HelpOak => "oak",
        BanditsChoice::HelpKraityn => "kraityn",
    }
}

fn is_page_allowed(page: &GuidePage, settings: &LevelingGuideSettings) -> bool {
    let Some(condition) = page.condition.as_ref() else {
        return true;
    };

    match condition {
        GuideCondition::LeagueStart { enabled } => settings.league_start == *enabled,
        GuideCondition::Bandit { allowed } => allowed.iter().any(|key| key == bandit_key(settings)),
        GuideCondition::OptionalQuests { enabled } => settings.optional_quests == *enabled,
        GuideCondition::LevelRecommendations { enabled } => {
            settings.level_recommendations == *enabled
        }
    }
}

fn eligible_page_indices(act: &GuideAct, settings: &LevelingGuideSettings) -> Vec<usize> {
    act.iter()
        .enumerate()
        .filter(|(_, page)| is_page_allowed(page, settings))
        .map(|(index, _)| index)
        .collect()
}

fn first_eligible_page_in_act(act: &GuideAct, settings: &LevelingGuideSettings) -> Option<usize> {
    act.iter()
        .enumerate()
        .find(|(_, page)| is_page_allowed(page, settings))
        .map(|(index, _)| index)
}

fn last_eligible_page_in_act(act: &GuideAct, settings: &LevelingGuideSettings) -> Option<usize> {
    act.iter()
        .enumerate()
        .rev()
        .find(|(_, page)| is_page_allowed(page, settings))
        .map(|(index, _)| index)
}

pub(crate) fn next_position(
    guide: &GuideData,
    position: GuidePosition,
    settings: &LevelingGuideSettings,
) -> GuidePosition {
    if guide.is_empty() {
        return GuidePosition::start();
    }

    let mut act_index = position.act_index.min(guide.len().saturating_sub(1));
    let page_index = position.page_index;

    if let Some(act) = guide.get(act_index) {
        for index in (page_index + 1)..act.len() {
            if is_page_allowed(&act[index], settings) {
                return GuidePosition {
                    act_index,
                    page_index: index,
                };
            }
        }
    }

    act_index += 1;
    while act_index < guide.len() {
        if let Some(act) = guide.get(act_index) {
            if let Some(first) = first_eligible_page_in_act(act, settings) {
                return GuidePosition {
                    act_index,
                    page_index: first,
                };
            }
        }
        act_index += 1;
    }

    GuidePosition {
        act_index: position.act_index,
        page_index: position.page_index,
    }
}

pub(crate) fn previous_position(
    guide: &GuideData,
    position: GuidePosition,
    settings: &LevelingGuideSettings,
) -> GuidePosition {
    if guide.is_empty() {
        return GuidePosition::start();
    }

    let mut act_index = position.act_index.min(guide.len().saturating_sub(1));
    let page_index = position.page_index.min(
        guide
            .get(act_index)
            .map(|act| act.len().saturating_sub(1))
            .unwrap_or(0),
    );

    if let Some(act) = guide.get(act_index) {
        for index in (0..page_index).rev() {
            if is_page_allowed(&act[index], settings) {
                return GuidePosition {
                    act_index,
                    page_index: index,
                };
            }
        }
    }

    while act_index > 0 {
        act_index -= 1;
        if let Some(act) = guide.get(act_index) {
            if let Some(last) = last_eligible_page_in_act(act, settings) {
                return GuidePosition {
                    act_index,
                    page_index: last,
                };
            }
        }
    }

    GuidePosition {
        act_index: position.act_index,
        page_index: position.page_index,
    }
}

#[derive(Debug, Clone, Copy)]
struct BossHighlightState {
    previous_was_kill: bool,
    previous_was_arena_image: bool,
}

fn is_arena_followup_image_key(key: &str) -> bool {
    let normalized = key.trim().to_ascii_lowercase().replace(' ', "_");
    normalized == "arena" || normalized == "in-out2"
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

    if token
        .chars()
        .last()
        .is_some_and(|ch: char| ch.is_ascii_digit())
    {
        token.pop();
    }

    (true, token)
}

fn css_color_from_tag(color: &str) -> String {
    let trimmed = color.trim();
    if trimmed.starts_with('#') {
        return trimmed.to_string();
    }

    let is_hex = !trimmed.is_empty() && trimmed.chars().all(|c: char| c.is_ascii_hexdigit());

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

        let is_trial_or_lab = {
            let probe = text.to_ascii_lowercase();
            probe.contains("trial") || probe.contains("_lab")
        };

        if text.contains('_') {
            text = text.replace('_', " ");
        }

        let has_arena_prefix = text.contains("arena:");
        if has_arena_prefix {
            text = text.replace("arena:", "");
        }

        if text.is_empty() {
            continue;
        }

        let arena_followup_highlight = state.previous_was_arena_image && text.trim() != ",";
        state.previous_was_arena_image = false;

        let comparison = normalize_token_for_comparison(&text);

        let excluded_from_kill = comparison == "everything" || comparison == "it";

        let boss_highlight =
            (state.previous_was_kill && !excluded_from_kill && !comparison.is_empty())
                || (has_arena_prefix && !comparison.is_empty());

        let hint_data_uri = match hint_key.as_deref() {
            Some(key) => {
                load_hint_data_uri_cached(app, hint_image_path_by_key, hint_image_cache, key)?
            }
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
            .or_else(|| arena_followup_highlight.then(|| ARENA_IMAGE_FOLLOWUP_COLOR.to_string()))
            .or_else(|| boss_highlight.then(|| BOSS_TARGET_COLOR.to_string()))
            .or_else(|| quest_reference.then(|| QUEST_REFERENCE_COLOR.to_string()))
            .or_else(|| quest_item.as_ref().map(|_| QUEST_ITEM_COLOR.to_string()))
            .or_else(|| is_trial_or_lab.then(|| TRIAL_OR_LAB_COLOR.to_string()));

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
        previous_was_arena_image: false,
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
        highlight_state.previous_was_arena_image = is_arena_followup_image_key(key);
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
    settings: &LevelingGuideSettings,
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
    let stripped = apply_level_recommendation_setting(stripped, settings);

    let spans = render_spans(
        app,
        guide_path,
        icon_cache,
        hint_keys,
        hint_image_path_by_key,
        hint_image_cache,
        &stripped,
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

    acts.iter()
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

                        return Ok(GuidePage {
                            lines: parsed_lines,
                            condition: None,
                        });
                    }

                    if let Some(object) = page_value.as_object() {
                        let condition = object
                            .get("condition")
                            .map(|value| parse_guide_condition(value))
                            .transpose()?;

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

                            return Ok(GuidePage {
                                lines: parsed_lines,
                                condition,
                            });
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

fn parse_guide_condition(value: &serde_json::Value) -> Result<GuideCondition, CommandError> {
    let array = value.as_array().ok_or_else(|| {
        command_error("guide_invalid_format", "Expected condition to be an array")
    })?;
    if array.len() != 2 {
        return Err(command_error(
            "guide_invalid_format",
            "Expected condition to have exactly 2 elements",
        ));
    }

    let key = array[0].as_str().ok_or_else(|| {
        command_error(
            "guide_invalid_format",
            "Expected condition key to be a string",
        )
    })?;

    match key {
        "league-start" => {
            let value = array[1].as_str().ok_or_else(|| {
                command_error(
                    "guide_invalid_format",
                    "Expected league-start condition value",
                )
            })?;
            let enabled = match value {
                "yes" => true,
                "no" => false,
                _ => {
                    return Err(command_error(
                        "guide_invalid_format",
                        "Expected league-start condition to be yes/no",
                    ))
                }
            };
            Ok(GuideCondition::LeagueStart { enabled })
        }
        "bandit" => {
            let allowed = array[1].as_array().ok_or_else(|| {
                command_error(
                    "guide_invalid_format",
                    "Expected bandit condition to be an array",
                )
            })?;
            let allowed = allowed
                .iter()
                .map(|value| {
                    value.as_str().map(str::to_string).ok_or_else(|| {
                        command_error(
                            "guide_invalid_format",
                            "Expected bandit condition values to be strings",
                        )
                    })
                })
                .collect::<Result<Vec<String>, CommandError>>()?;
            Ok(GuideCondition::Bandit { allowed })
        }
        "optional-quests" => {
            let value = array[1].as_str().ok_or_else(|| {
                command_error(
                    "guide_invalid_format",
                    "Expected optional-quests condition value",
                )
            })?;
            let enabled = match value {
                "yes" => true,
                "no" => false,
                _ => {
                    return Err(command_error(
                        "guide_invalid_format",
                        "Expected optional-quests condition to be yes/no",
                    ))
                }
            };
            Ok(GuideCondition::OptionalQuests { enabled })
        }
        "level-recommendations" => {
            let value = array[1].as_str().ok_or_else(|| {
                command_error(
                    "guide_invalid_format",
                    "Expected level-recommendations condition value",
                )
            })?;
            let enabled = match value {
                "yes" => true,
                "no" => false,
                _ => {
                    return Err(command_error(
                        "guide_invalid_format",
                        "Expected level-recommendations condition to be yes/no",
                    ))
                }
            };
            Ok(GuideCondition::LevelRecommendations { enabled })
        }
        _ => Err(command_error(
            "guide_invalid_format",
            format!("Unknown condition key: {key}"),
        )),
    }
}

pub(crate) fn clamp_position(
    guide: &GuideData,
    position: GuidePosition,
    settings: &LevelingGuideSettings,
) -> GuidePosition {
    if guide.is_empty() {
        return GuidePosition::start();
    }

    let act_index = position.act_index.min(guide.len().saturating_sub(1));
    let mut clamped = GuidePosition {
        act_index,
        page_index: position.page_index,
    };

    if let Some(act) = guide.get(clamped.act_index) {
        if act.is_empty() {
            clamped.page_index = 0;
        } else {
            clamped.page_index = clamped.page_index.min(act.len().saturating_sub(1));
        }
    }

    if guide
        .get(clamped.act_index)
        .and_then(|act| act.get(clamped.page_index))
        .is_some_and(|page| is_page_allowed(page, settings))
    {
        return clamped;
    }

    let forward = next_position(guide, clamped, settings);
    if forward != clamped {
        return forward;
    }

    let backward = previous_position(guide, clamped, settings);
    if backward != clamped {
        return backward;
    }

    for (act_index, act) in guide.iter().enumerate() {
        if let Some(first) = first_eligible_page_in_act(act, settings) {
            return GuidePosition {
                act_index,
                page_index: first,
            };
        }
    }

    GuidePosition::start()
}

pub(crate) fn current_page_dto(
    app: &AppHandle,
    loaded: &mut LoadedGuide,
    settings: &LevelingGuideSettings,
) -> Result<LevelingGuidePageDto, CommandError> {
    loaded.position = clamp_position(&loaded.guide, loaded.position, settings);

    let act = loaded
        .guide
        .get(loaded.position.act_index)
        .ok_or_else(|| command_error("guide_position_invalid", "Act index out of bounds"))?;

    let eligible = eligible_page_indices(act, settings);
    let page_count_in_act = eligible.len();
    let display_page_index = eligible
        .iter()
        .position(|index| *index == loaded.position.page_index)
        .unwrap_or(0);

    let page = act
        .get(loaded.position.page_index)
        .ok_or_else(|| command_error("guide_position_invalid", "Page index out of bounds"))?;

    let has_previous =
        previous_position(&loaded.guide, loaded.position, settings) != loaded.position;
    let has_next = next_position(&loaded.guide, loaded.position, settings) != loaded.position;

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
                settings,
                line,
            )
        })
        .collect::<Result<Vec<Option<LevelingGuideLineDto>>, CommandError>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<LevelingGuideLineDto>>();

    let target_area_id = extract_last_area_id_from_page(page);
    let target_area = target_area_id
        .as_deref()
        .and_then(|id| loaded.area_name_by_id.get(id).cloned());

    loaded.target_area_id = target_area_id.clone();

    let mut campaign_page_index: usize = 0;
    let mut campaign_page_count: usize = 0;
    for (act_index, act_pages) in loaded.guide.iter().enumerate() {
        let act_eligible_count = eligible_page_indices(act_pages, settings).len();
        campaign_page_count += act_eligible_count;
        if act_index < loaded.position.act_index {
            campaign_page_index += act_eligible_count;
        } else if act_index == loaded.position.act_index {
            campaign_page_index += display_page_index;
        }
    }

    Ok(LevelingGuidePageDto {
        guide_path: loaded.guide_path.clone(),
        position: GuidePosition {
            act_index: loaded.position.act_index,
            page_index: display_page_index,
        },
        act_count: loaded.guide.len(),
        page_count_in_act,
        lines,
        has_previous,
        has_next,
        campaign_page_index,
        campaign_page_count,
        target_area,
        target_area_id,
    })
}
