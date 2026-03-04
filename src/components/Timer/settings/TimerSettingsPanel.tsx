import { JSX } from 'react';

import '../../LevelingGuide/settings/LevelingGuideSettingsPanel.css';

type TimerSettingsPanelProps = {
	actTimerEnabled: boolean;
	campaignTimerEnabled: boolean;
	onActTimerEnabledChange: (value: boolean) => void;
	onCampaignTimerEnabledChange: (value: boolean) => void;
	settingsLoading: boolean;
};

export function TimerSettingsPanel(props: TimerSettingsPanelProps): JSX.Element {
	const { settingsLoading } = props;

	return (
		<div className="settingsPanel">
			<div className="settingsGroup">
				<div className="settingsGroupTitle">Timer Options</div>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.actTimerEnabled}
						onChange={(event) => props.onActTimerEnabledChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Act timer</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.campaignTimerEnabled}
						onChange={(event) => props.onCampaignTimerEnabledChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Campaign timer</span>
				</label>
			</div>
		</div>
	);
}
