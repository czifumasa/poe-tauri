use crate::error::CommandError;
use crate::persistence::store;
use tauri::AppHandle;

use super::types::PersistedLevelingGuideProgress;

const LEVELING_GUIDE_PROGRESS_KEY: &str = "leveling_guide_progress";

pub(crate) fn load_leveling_guide_progress(
    app: &AppHandle,
    default_relative_resource_path: &str,
) -> Result<PersistedLevelingGuideProgress, CommandError> {
    let maybe_progress =
        store::get_optional::<PersistedLevelingGuideProgress>(app, LEVELING_GUIDE_PROGRESS_KEY)?;

    let progress = maybe_progress.unwrap_or_else(|| {
        PersistedLevelingGuideProgress::default_for_resource(default_relative_resource_path)
    });

    Ok(progress)
}

pub(crate) fn has_persisted_leveling_guide_progress(
    app: &AppHandle,
) -> Result<bool, CommandError> {
    let maybe_progress =
        store::get_optional::<PersistedLevelingGuideProgress>(app, LEVELING_GUIDE_PROGRESS_KEY)?;
    Ok(maybe_progress.is_some())
}

pub(crate) fn save_leveling_guide_progress(
    app: &AppHandle,
    progress: &PersistedLevelingGuideProgress,
) -> Result<(), CommandError> {
    store::set_value(app, LEVELING_GUIDE_PROGRESS_KEY, progress)
}
