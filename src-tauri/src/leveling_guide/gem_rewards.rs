use std::collections::HashSet;

use super::gem_db::{attribute_color, GemDatabase, GemEntry};
use super::pob_parser::PobImportData;
use super::types::{GuideData, GuidePage};

fn format_colored_gem_label(gem: &str, gem_entry: &GemEntry) -> String {
    let display_name = gem_entry
        .name
        .as_deref()
        .unwrap_or(gem)
        .replace(" support", "")
        .replace(' ', "_");

    let color_tag = attribute_color(gem_entry.attribute)
        .map(|c| format!("(color:{c})"))
        .unwrap_or_default();

    format!("{color_tag}{display_name}")
}

pub(crate) fn inject_gem_rewards(
    guide: &mut GuideData,
    gem_db: &GemDatabase,
    pob: &PobImportData,
    league_start: bool,
) {
    let class = pob.class.to_ascii_lowercase();
    let mut wanted_gems = build_wanted_gems(pob, league_start);

    let lilly_quests: HashSet<&str> = ["mercy mission", "a fixture of fate", "fallen from grace"]
        .iter()
        .copied()
        .collect();

    for act in guide.iter_mut() {
        let page_count = act.len();
        let mut pages_to_remove: Vec<usize> = Vec::new();

        for page_idx in 0..page_count {
            let page = &act[page_idx];

            if !page_has_quest_reference(page) {
                continue;
            }

            let lines = page_lines(page);
            let mut new_lines: Vec<String> = Vec::new();
            let mut reward_available: HashSet<String> = HashSet::new();
            let mut vendor_available: HashSet<String> = HashSet::new();
            let mut quests_on_page: HashSet<String> = HashSet::new();
            let mut has_quest_check = false;

            for line in lines {
                if line.contains("quest-check") {
                    has_quest_check = true;
                    continue;
                }

                let quests_on_line = extract_quest_tags(line);
                for q in &quests_on_line {
                    quests_on_page.insert(q.to_ascii_lowercase());
                }

                if !line.contains('<') {
                    new_lines.push(line.to_string());
                    continue;
                }

                new_lines.push(line.to_string());

                let is_lilly_line = is_lilly_quest_line(line, &lilly_quests);
                if is_lilly_line {
                    continue;
                }

                let mut consumed_on_line = false;
                for quest in &quests_on_line {
                    if consumed_on_line {
                        break;
                    }
                    let quest_key = quest.to_ascii_lowercase();
                    let mut gem_idx = 0;
                    while gem_idx < wanted_gems.len() {
                        let gem = &wanted_gems[gem_idx];
                        let gem_entry = match gem_db.gems.get(gem.as_str()) {
                            Some(e) => e,
                            None => {
                                gem_idx += 1;
                                continue;
                            }
                        };

                        let quest_availability = match gem_entry.quests.get(quest_key.as_str()) {
                            Some(a) => a,
                            None => {
                                gem_idx += 1;
                                continue;
                            }
                        };

                        let can_get_as_reward = match &quest_availability.quest {
                            Some(classes) => classes.is_empty() || classes.contains(&class),
                            None => false,
                        };

                        if !can_get_as_reward {
                            gem_idx += 1;
                            continue;
                        }

                        let reward_line = format!(
                            "(hint)__ gem reward: {}",
                            format_colored_gem_label(gem.as_str(), gem_entry)
                        );
                        new_lines.push(reward_line);

                        reward_available.insert(quest_key.clone());

                        let is_quicksilver = gem == "quicksilver flask";
                        wanted_gems.remove(gem_idx);

                        if !is_quicksilver {
                            consumed_on_line = true;
                            break;
                        }
                    }
                }
            }

            if !quests_on_page.is_empty() {
                let mut vendor_gem_indices: Vec<usize> = Vec::new();
                for (gem_idx, gem) in wanted_gems.iter().enumerate() {
                    let gem_entry = match gem_db.gems.get(gem.as_str()) {
                        Some(e) => e,
                        None => continue,
                    };

                    let mut best_candidate: Option<(usize, String)> = None;
                    let mut should_defer = false;

                    for (quest_name, availability) in &gem_entry.quests {
                        let quest_key = quest_name.to_ascii_lowercase();
                        if !quests_on_page.contains(&quest_key) {
                            continue;
                        }

                        let can_buy = match &availability.vendor {
                            Some(classes) => classes.is_empty() || classes.contains(&class),
                            None => false,
                        };

                        if !can_buy {
                            continue;
                        }

                        if has_quest_check {
                            let next_page = act.get(page_idx + 1);
                            if let Some(next) = next_page {
                                let next_has_quest_check =
                                    page_lines(next).iter().any(|l| l.contains("quest-check"));
                                let same_condition =
                                    condition_matches(&page.condition, &next.condition);
                                if next_has_quest_check && same_condition {
                                    should_defer = true;
                                    break;
                                }
                            }
                        }

                        let insert_pos = find_insert_position_after_quest(&new_lines, &quest_key)
                            .unwrap_or_else(|| find_vendor_insert_position(&new_lines));

                        match &best_candidate {
                            None => best_candidate = Some((insert_pos, quest_key)),
                            Some((best_pos, _)) if insert_pos < *best_pos => {
                                best_candidate = Some((insert_pos, quest_key))
                            }
                            _ => {}
                        }
                    }

                    if should_defer {
                        continue;
                    }

                    if let Some((insert_pos, quest_key)) = best_candidate {
                        vendor_available.insert(quest_key);
                        new_lines.insert(
                            insert_pos,
                            format!(
                                "(hint)__ buy gem: {}",
                                format_colored_gem_label(gem.as_str(), gem_entry)
                            ),
                        );
                        vendor_gem_indices.push(gem_idx);
                    }
                }

                for idx in vendor_gem_indices.into_iter().rev() {
                    wanted_gems.remove(idx);
                }
            }

            wanted_gems.retain(|g| !g.is_empty());

            if has_quest_check && reward_available.is_empty() && vendor_available.is_empty() {
                pages_to_remove.push(page_idx);
            } else {
                prune_empty_quest_lines(
                    &mut new_lines,
                    gem_db,
                    &reward_available,
                    &vendor_available,
                    &lilly_quests,
                );
                set_page_lines(&mut act[page_idx], new_lines);
            }
        }

        for idx in pages_to_remove.into_iter().rev() {
            act.remove(idx);
        }
    }
}

pub(crate) fn prune_gem_quest_lines(guide: &mut GuideData, gem_db: &GemDatabase) {
    for act in guide.iter_mut() {
        let mut pages_to_remove: Vec<usize> = Vec::new();

        for page_idx in 0..act.len() {
            let page = &act[page_idx];
            let lines = page_lines(page);
            let mut new_lines: Vec<String> = Vec::new();

            for line in lines {
                if line.contains("quest-check") {
                    continue;
                }

                if !line.contains(": <") {
                    new_lines.push(line.to_string());
                    continue;
                }

                let quests_on_line = extract_quest_tags(line);
                let is_gem_quest_line = quests_on_line
                    .iter()
                    .any(|q| gem_db.quests.contains_key(q.to_ascii_lowercase().as_str()));

                if !is_gem_quest_line {
                    new_lines.push(line.to_string());
                }
            }

            if new_lines.is_empty() {
                pages_to_remove.push(page_idx);
            } else {
                set_page_lines(&mut act[page_idx], new_lines);
            }
        }

        for idx in pages_to_remove.into_iter().rev() {
            act.remove(idx);
        }
    }
}

fn build_wanted_gems(pob: &PobImportData, league_start: bool) -> Vec<String> {
    let mut gems: Vec<String> = if league_start {
        vec!["quicksilver flask".to_string()]
    } else {
        Vec::new()
    };

    for gem_name in &pob.gem_names {
        let lower = gem_name.to_ascii_lowercase();
        if !gems.contains(&lower) {
            gems.push(lower);
        }
    }

    gems
}

fn page_has_quest_reference(page: &GuidePage) -> bool {
    page.lines.iter().any(|line| line.contains(": <"))
}

fn page_lines(page: &GuidePage) -> &[String] {
    &page.lines
}

fn set_page_lines(page: &mut GuidePage, lines: Vec<String>) {
    page.lines = lines;
}

fn extract_quest_tags(line: &str) -> Vec<String> {
    let mut quests = Vec::new();
    let mut search_from = 0;
    while let Some(open) = line[search_from..].find('<') {
        let abs_open = search_from + open;
        let rest = &line[abs_open + 1..];
        if let Some(close) = rest.find('>') {
            let tag = &rest[..close];
            let quest_name = tag.replace('_', " ");
            quests.push(quest_name);
            search_from = abs_open + 1 + close + 1;
        } else {
            break;
        }
    }
    quests
}

fn is_lilly_quest_line(line: &str, lilly_quests: &HashSet<&str>) -> bool {
    let lower = line.to_ascii_lowercase();
    if !lower.contains("lilly") {
        return false;
    }

    let quests_on_line = extract_quest_tags(line);
    for q in &quests_on_line {
        let q_lower = q.to_ascii_lowercase();
        if !lilly_quests.contains(q_lower.as_str()) {
            return false;
        }
    }

    true
}

fn find_vendor_insert_position(lines: &[String]) -> usize {
    let mut last_quest_line_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(": <") {
            last_quest_line_idx = Some(i);
        }
    }

    let start = match last_quest_line_idx {
        Some(idx) => idx + 1,
        None => return lines.len(),
    };

    let mut pos = start;
    while pos < lines.len() {
        let lower = lines[pos].to_ascii_lowercase();
        if lower.starts_with("(hint)") || lower.starts_with("(img:quest)") {
            pos += 1;
        } else {
            break;
        }
    }

    pos
}

fn find_insert_position_after_quest(lines: &[String], quest_key: &str) -> Option<usize> {
    let quest_token = format!("<{}>", quest_key.replace(' ', "_"));
    let mut last_match_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.contains(&quest_token) {
            last_match_idx = Some(i);
        }
    }

    let start = last_match_idx? + 1;
    let mut pos = start;
    while pos < lines.len() {
        let lower = lines[pos].to_ascii_lowercase();
        if lower.starts_with("(hint)") {
            pos += 1;
        } else {
            break;
        }
    }

    Some(pos)
}

fn cleanup_pruned_quest_line(line: &str) -> String {
    let mut s = line.to_string();
    while s.contains(", ,") {
        s = s.replace(", ,", ", ");
    }
    s = s.replace(": ,", ":");
    s = s.replace(" ,", ",");
    while s.contains("  ") {
        s = s.replace("  ", " ");
    }
    s = s.replace(":  ", ": ");
    s.trim().trim_end_matches(',').trim().to_string()
}

fn condition_matches(
    a: &Option<super::types::GuideCondition>,
    b: &Option<super::types::GuideCondition>,
) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(ac), Some(bc)) => format!("{ac:?}") == format!("{bc:?}"),
        _ => false,
    }
}

fn prune_empty_quest_lines(
    lines: &mut Vec<String>,
    gem_db: &GemDatabase,
    reward_available: &HashSet<String>,
    vendor_available: &HashSet<String>,
    lilly_quests: &HashSet<&str>,
) {
    let mut indices_to_remove: Vec<usize> = Vec::new();

    for (index, line) in lines.iter_mut().enumerate() {
        let lower = line.to_ascii_lowercase();
        if let Some(split_pos) = lower.find(" || lilly:") {
            *line = line[..split_pos].to_string();
        }

        let quests_on_line_raw = extract_quest_tags(line);
        let quests_on_line: Vec<String> = quests_on_line_raw
            .into_iter()
            .filter(|q| gem_db.quests.contains_key(q.to_ascii_lowercase().as_str()))
            .collect();

        if quests_on_line.is_empty() {
            continue;
        }

        if is_lilly_quest_line(line, lilly_quests) {
            continue;
        }

        let mut skipped: Vec<String> = Vec::new();
        let mut relevant_count = 0usize;

        for quest in &quests_on_line {
            let quest_key = quest.to_ascii_lowercase();
            let is_relevant =
                reward_available.contains(&quest_key) || vendor_available.contains(&quest_key);
            if is_relevant {
                relevant_count += 1;
            } else {
                skipped.push(quest_key);
            }
        }

        if relevant_count == 0 {
            indices_to_remove.push(index);
            continue;
        }

        for quest_key in skipped {
            let token = format!("<{}>", quest_key.replace(' ', "_"));
            if line.contains(&token) {
                *line = line.replace(&token, "");
            }
        }

        *line = cleanup_pruned_quest_line(line);
    }

    for idx in indices_to_remove.into_iter().rev() {
        lines.remove(idx);
    }
}
