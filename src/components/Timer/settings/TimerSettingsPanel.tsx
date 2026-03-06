import { JSX } from 'react';

import '../../LevelingGuide/settings/LevelingGuideSettingsPanel.css';

type TimerSettingsPanelProps = {
	enabled: boolean;
	displayActTimer: boolean;
	displayCampaignTimer: boolean;
	warnWhenPaused: boolean;
	onEnabledChange: (value: boolean) => void;
	onDisplayActTimerChange: (value: boolean) => void;
	onDisplayCampaignTimerChange: (value: boolean) => void;
	onWarnWhenPausedChange: (value: boolean) => void;
	settingsLoading: boolean;
};

export function TimerSettingsPanel(props: TimerSettingsPanelProps): JSX.Element {
	const { enabled, settingsLoading } = props;

	return (
		<div className="settingsPanel">
			<div className="settingsPanelDescription">Track act and campaign completion times.</div>

			<div className="settingsGroup">
				<div className="settingsGroupTitle">Timer Options</div>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={enabled}
						onChange={(event) => props.onEnabledChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Enable timer</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.displayActTimer}
						onChange={(event) => props.onDisplayActTimerChange(event.currentTarget.checked)}
						disabled={settingsLoading || !enabled}
					/>
					<span className="settingsToggleLabel">Display act timer in leveling guide</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.displayCampaignTimer}
						onChange={(event) => props.onDisplayCampaignTimerChange(event.currentTarget.checked)}
						disabled={settingsLoading || !enabled}
					/>
					<span className="settingsToggleLabel">Display campaign timer in leveling guide</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.warnWhenPaused}
						onChange={(event) => props.onWarnWhenPausedChange(event.currentTarget.checked)}
						disabled={settingsLoading || !enabled}
					/>
					<span className="settingsToggleLabel">Warn when paused</span>
				</label>
			</div>
		</div>
	);
}
