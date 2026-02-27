import { JSX } from 'react';
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
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
};

const banditsChoiceOptions: ReadonlyArray<{ value: BanditsChoice; label: string }> = [
	{ value: 'KillAll', label: 'Kill all' },
	{ value: 'HelpAlira', label: 'Help Alira' },
	{ value: 'HelpOak', label: 'Help Oak' },
	{ value: 'HelpKraityn', label: 'Help Kraityn' },
];

function getDashboardHeaderLabel(page: LevelingGuidePageDto): string {
	const actLabel = `Act ${page.position.actIndex + 1}`;
	return `${actLabel} - Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;
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
