import { JSX } from 'react';
import type { BanditsChoice } from '../../../types/Settings.ts';

import './LevelingGuideSettingsPanel.css';

type BanditsOption = { readonly value: BanditsChoice; readonly label: string };

const BANDITS_OPTIONS: readonly BanditsOption[] = [
	{ value: 'KillAll', label: 'Kill All' },
	{ value: 'HelpAlira', label: 'Help Alira' },
	{ value: 'HelpOak', label: 'Help Oak' },
	{ value: 'HelpKraityn', label: 'Help Kraityn' },
] as const;

type LevelingGuideSettingsPanelProps = {
	settingsLoading: boolean;
	leagueStart: boolean;
	onLeagueStartChange: (value: boolean) => Promise<void>;
	optionalQuests: boolean;
	onOptionalQuestsChange: (value: boolean) => Promise<void>;
	levelRecommendations: boolean;
	onLevelRecommendationsChange: (value: boolean) => Promise<void>;
	banditsChoice: BanditsChoice;
	onBanditsChoiceChange: (value: BanditsChoice) => Promise<void>;
	clientLogPath: string | null;
	onClientLogPathBrowse: () => Promise<void>;
	onClientLogPathClear: () => Promise<void>;
	gemsEnabled: boolean;
	onGemsEnabledChange: (value: boolean) => Promise<void>;
};

function formatLogPathDisplay(path: string | null): string {
	if (path === null) {
		return 'Not configured';
	}
	const maxLength = 40;
	if (path.length <= maxLength) {
		return path;
	}
	return `…${path.slice(-maxLength)}`;
}

export function LevelingGuideSettingsPanel(props: LevelingGuideSettingsPanelProps): JSX.Element {
	const { settingsLoading } = props;

	return (
		<div className="settingsPanel">
			<div className="settingsPanelDescription">Step-by-step act progression with gem and quest tracking.</div>

			<div className="settingsGroup">
				<div className="settingsGroupTitle">Guide Options</div>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.leagueStart}
						onChange={(event) => void props.onLeagueStartChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">League start</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.optionalQuests}
						onChange={(event) => void props.onOptionalQuestsChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Optional quests</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.levelRecommendations}
						onChange={(event) => void props.onLevelRecommendationsChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Level recommendations</span>
				</label>

				<label className="settingsToggle">
					<input
						type="checkbox"
						checked={props.gemsEnabled}
						onChange={(event) => void props.onGemsEnabledChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					<span className="settingsToggleLabel">Gems</span>
				</label>
			</div>

			<div className="settingsGroup">
				<div className="settingsGroupTitle">Bandits</div>
				<select
					className="settingsSelect"
					value={props.banditsChoice}
					onChange={(event) => {
						const nextRaw = event.currentTarget.value;
						const next = BANDITS_OPTIONS.find((opt) => opt.value === nextRaw)?.value;
						if (next !== undefined) {
							void props.onBanditsChoiceChange(next);
						}
					}}
					disabled={settingsLoading}>
					{BANDITS_OPTIONS.map((option) => (
						<option key={option.value} value={option.value}>
							{option.label}
						</option>
					))}
				</select>
			</div>

			<div className="settingsGroup">
				<div className="settingsGroupTitle">Client Log</div>
				<div className="settingsClientLogRow">
					<span className="settingsClientLogPath" title={props.clientLogPath ?? ''}>
						{formatLogPathDisplay(props.clientLogPath)}
					</span>
					<button
						type="button"
						className="settingsClientLogButton"
						onClick={() => void props.onClientLogPathBrowse()}
						disabled={settingsLoading}>
						Browse
					</button>
					{props.clientLogPath !== null && (
						<button
							type="button"
							className="settingsClientLogButton"
							onClick={() => void props.onClientLogPathClear()}
							disabled={settingsLoading}>
							Clear
						</button>
					)}
				</div>
			</div>
		</div>
	);
}
