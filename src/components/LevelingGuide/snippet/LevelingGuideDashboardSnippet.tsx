import { JSX, useCallback, useState } from 'react';
import type { LevelingGuidePageDto } from '../../../types/Guide.ts';
import type { BanditsChoice } from '../../../types/Settings.ts';

import '../LevelingGuideCommon.css';
import './LevelingGuideDashboardSnippet.css';

type LevelingGuideDashboardSnippetProps = {
	page: LevelingGuidePageDto | null;
	loading: boolean;
	settingsLoading: boolean;
	error: string | null;
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
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
	onImportPob: (pobCode: string) => Promise<void>;
	pobClass: string | null;
	pobGemCount: number | null;
};

function formatLogPathDisplay(path: string | null): string {
	if (path === null || path === '') {
		return 'Not set';
	}
	const maxLength = 40;
	if (path.length <= maxLength) {
		return path;
	}
	return `…${path.slice(-(maxLength - 1))}`;
}

const banditsChoiceOptions: ReadonlyArray<{ value: BanditsChoice; label: string }> = [
	{ value: 'KillAll', label: 'Kill all' },
	{ value: 'HelpAlira', label: 'Help Alira' },
	{ value: 'HelpOak', label: 'Help Oak' },
	{ value: 'HelpKraityn', label: 'Help Kraityn' },
];

function getDashboardHeaderLabel(page: LevelingGuidePageDto): string {
	const actLabel = `Act ${page.position.actIndex + 1}`;
	const pageLabel = `Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;
	if (page.targetArea) {
		return `${actLabel} - ${pageLabel} — ${page.targetArea}`;
	}
	return `${actLabel} - ${pageLabel}`;
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
		<div className="pobImportSection">
			<div className="pobImportStatus">{statusLabel}</div>
			<div className="pobImportRow">
				<input
					type="text"
					className="pobImportInput"
					placeholder="Paste PoB export code"
					value={pobInput}
					onChange={(event) => setPobInput(event.currentTarget.value)}
					disabled={props.disabled || importing}
				/>
				<button
					type="button"
					className="pobImportButton"
					onClick={() => void handleImport()}
					disabled={props.disabled || importing || pobInput.trim() === ''}>
					{importing ? 'Importing\u2026' : 'Import'}
				</button>
			</div>
			{importError !== null && <div className="pobImportError">{importError}</div>}
		</div>
	);
}

export function LevelingGuideDashboardSnippet(props: LevelingGuideDashboardSnippetProps): JSX.Element {
	const { page, loading, settingsLoading } = props;
	if (page === null) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{props.error && <div className="overlayError">{props.error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
				<div className="guideDashboardSettings">
					<label className="leaguestartToggle">
						<input
							type="checkbox"
							checked={props.leagueStart}
							onChange={(event) => void props.onLeagueStartChange(event.currentTarget.checked)}
							disabled={settingsLoading}
						/>
						League start
					</label>
					<label className="leaguestartToggle">
						<input
							type="checkbox"
							checked={props.optionalQuests}
							onChange={(event) => void props.onOptionalQuestsChange(event.currentTarget.checked)}
							disabled={settingsLoading}
						/>
						Optional quests
					</label>
					<label className="leaguestartToggle">
						<input
							type="checkbox"
							checked={props.levelRecommendations}
							onChange={(event) => void props.onLevelRecommendationsChange(event.currentTarget.checked)}
							disabled={settingsLoading}
						/>
						Level recommendations
					</label>
					<label className="leaguestartToggle">
						<span>Bandits</span>
						<select
							value={props.banditsChoice}
							onChange={(event) => {
								const nextRaw = event.currentTarget.value;
								const next = banditsChoiceOptions.find((opt) => opt.value === nextRaw)?.value;
								if (next !== undefined) {
									void props.onBanditsChoiceChange(next);
								}
							}}
							disabled={settingsLoading}>
							{banditsChoiceOptions.map((option) => (
								<option key={option.value} value={option.value}>
									{option.label}
								</option>
							))}
						</select>
					</label>
					<div className="clientLogPathRow">
						<span className="clientLogPathLabel">Client.txt</span>
						<span className="clientLogPathValue" title={props.clientLogPath ?? ''}>
							{formatLogPathDisplay(props.clientLogPath)}
						</span>
						<button
							type="button"
							className="clientLogPathButton"
							onClick={() => void props.onClientLogPathBrowse()}
							disabled={settingsLoading}>
							Browse
						</button>
						{props.clientLogPath !== null && (
							<button
								type="button"
								className="clientLogPathButton"
								onClick={() => void props.onClientLogPathClear()}
								disabled={settingsLoading}>
								Clear
							</button>
						)}
					</div>
					<label className="leaguestartToggle">
						<input
							type="checkbox"
							checked={props.gemsEnabled}
							onChange={(event) => void props.onGemsEnabledChange(event.currentTarget.checked)}
							disabled={settingsLoading}
						/>
						Gems
					</label>
					<PobImportSection
						onImportPob={props.onImportPob}
						pobClass={props.pobClass}
						pobGemCount={props.pobGemCount}
						disabled={settingsLoading}
					/>
				</div>
				<div className="guideDashboardControls">
					<button type="button" className="loadGuideButton" onClick={() => void props.onLoadGuide()} disabled={loading}>
						Load Guide
					</button>
					<button type="button" onClick={() => void props.onResetProgress()} disabled>
						Reset
					</button>
				</div>
			</div>
		);
	}

	return (
		<div className="guideContent guideContentCompact">
			<div className="guideHeader">{getDashboardHeaderLabel(page)}</div>
			<div className="guideDashboardSettings">
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.leagueStart}
						onChange={(event) => void props.onLeagueStartChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					League start
				</label>
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.optionalQuests}
						onChange={(event) => void props.onOptionalQuestsChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					Optional quests
				</label>
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.levelRecommendations}
						onChange={(event) => void props.onLevelRecommendationsChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					Level recommendations
				</label>
				<label className="leaguestartToggle">
					<span>Bandits</span>
					<select
						value={props.banditsChoice}
						onChange={(event) => {
							const nextRaw = event.currentTarget.value;
							const next = banditsChoiceOptions.find((opt) => opt.value === nextRaw)?.value;
							if (next !== undefined) {
								void props.onBanditsChoiceChange(next);
							}
						}}
						disabled={settingsLoading}>
						{banditsChoiceOptions.map((option) => (
							<option key={option.value} value={option.value}>
								{option.label}
							</option>
						))}
					</select>
				</label>
				<div className="clientLogPathRow">
					<span className="clientLogPathLabel">Client.txt</span>
					<span className="clientLogPathValue" title={props.clientLogPath ?? ''}>
						{formatLogPathDisplay(props.clientLogPath)}
					</span>
					<button
						type="button"
						className="clientLogPathButton"
						onClick={() => void props.onClientLogPathBrowse()}
						disabled={settingsLoading}>
						Browse
					</button>
					{props.clientLogPath !== null && (
						<button
							type="button"
							className="clientLogPathButton"
							onClick={() => void props.onClientLogPathClear()}
							disabled={settingsLoading}>
							Clear
						</button>
					)}
				</div>
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.gemsEnabled}
						onChange={(event) => void props.onGemsEnabledChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					Gems
				</label>
				<PobImportSection
					onImportPob={props.onImportPob}
					pobClass={props.pobClass}
					pobGemCount={props.pobGemCount}
					disabled={settingsLoading}
				/>
			</div>
			<div className="guideNavigation">
				<button type="button" onClick={() => void props.onLoadGuide()} disabled={loading}>
					Load
				</button>
				<button type="button" onClick={() => void props.onResetProgress()} disabled={loading}>
					Reset
				</button>
			</div>
		</div>
	);
}
