import { JSX } from 'react';
import type { LevelingGuidePageDto } from '../../../types/Guide.ts';

import '../LevelingGuideCommon.css';
import './LevelingGuideDashboardSnippet.css';

type LevelingGuideDashboardSnippetProps = {
	page: LevelingGuidePageDto | null;
	loading: boolean;
	settingsLoading: boolean;
	error: string | null;
	leaguestart: boolean;
	onLeaguestartChange: (value: boolean) => Promise<void>;
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
};

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
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.leaguestart}
						onChange={(event) => void props.onLeaguestartChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					Leaguestart
				</label>
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
			<div className="guideNavigation">
				<label className="leaguestartToggle">
					<input
						type="checkbox"
						checked={props.leaguestart}
						onChange={(event) => void props.onLeaguestartChange(event.currentTarget.checked)}
						disabled={settingsLoading}
					/>
					Leaguestart
				</label>
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
