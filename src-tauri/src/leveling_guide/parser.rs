use crate::error::{command_error, CommandError};

use super::types::{GuideAct, GuideData, GuidePage, GuidePosition, LevelingGuidePageDto, LoadedGuide};

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

pub(crate) fn current_page_dto(loaded: &LoadedGuide) -> Result<LevelingGuidePageDto, CommandError> {
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

    Ok(LevelingGuidePageDto {
        guide_path: loaded.guide_path.clone(),
        position: loaded.position,
        act_count: loaded.guide.len(),
        page_count_in_act: act.len(),
        lines: page.lines.clone(),
        has_previous,
        has_next,
    })
}
