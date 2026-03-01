import { JSX, useCallback, useState } from 'react';
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
	onImportPob: (pobCode: string) => Promise<void>;
	pobClass: string | null;
	pobGemCount: number | null;
	onBack: () => void;
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

function PobImportSection(props: {
	onImportPob: (pobCode: string) => Promise<void>;
	pobClass: string | null;
	pobGemCount: number | null;
	disabled: boolean;
}): JSX.Element {
	const [pobInput, setPobInput] = useState<string>('');
	const [importing, setImporting] = useState<boolean>(false);
	const [importError, setImportError] = useState<string | null>(null);

	const handleImport = useCallback(async (): Promise<void> => {
		const trimmed = pobInput.trim();
		if (trimmed === '') {
			return;
		}
		setImporting(true);
		setImportError(null);
		try {
			await props.onImportPob(trimmed);
			setPobInput('');
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			setImportError(message);
		} finally {
			setImporting(false);
		}
	}, [pobInput, props]);

	const statusLabel =
		props.pobClass !== null && props.pobGemCount !== null
			? `PoB: ${props.pobClass} (${props.pobGemCount} gems)`
			: 'No PoB imported';

	return (
		<div className="settingsPobSection">
			<div className="settingsPobStatus">{statusLabel}</div>
			<div className="settingsPobRow">
				<input
					type="text"
					className="settingsPobInput"
					placeholder="Paste PoB export code"
					value={pobInput}
					onChange={(event) => setPobInput(event.currentTarget.value)}
					disabled={props.disabled || importing}
				/>
				<button
					type="button"
					className="settingsPobButton"
					onClick={() => void handleImport()}
					disabled={props.disabled || importing || pobInput.trim() === ''}>
					{importing ? 'Importing\u2026' : 'Import'}
				</button>
			</div>
			{importError !== null && <div className="settingsPobError">{importError}</div>}
		</div>
	);
}

export function LevelingGuideSettingsPanel(props: LevelingGuideSettingsPanelProps): JSX.Element {
	const { settingsLoading } = props;

	return (
		<div className="settingsPanel">
			<div className="settingsPanelHeader">
				<button type="button" className="settingsBackButton" onClick={props.onBack}>
					← Back
				</button>
				<span className="settingsPanelTitle">Leveling Guide Settings</span>
			</div>

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

			<div className="settingsGroup">
				<div className="settingsGroupTitle">Path of Building</div>
				<PobImportSection
					onImportPob={props.onImportPob}
					pobClass={props.pobClass}
					pobGemCount={props.pobGemCount}
					disabled={settingsLoading}
				/>
			</div>
		</div>
	);
}
