import { JSX } from 'react';
import type { LevelingGuidePageDto } from '../../../types/Guide.ts';
import { ModuleSnippet } from '../../ModuleSnippet/ModuleSnippet.tsx';

import './LevelingGuideDashboardSnippet.css';

type LevelingGuideDashboardSnippetProps = {
	page: LevelingGuidePageDto | null;
	overlayVisible: boolean;
	loading: boolean;
	error: string | null;
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
	onShowOverlay: () => Promise<void>;
	onHideOverlay: () => Promise<void>;
	onOpenSettings: () => void;
};

function formatProgress(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

type GuideLoadedBodyProps = {
	page: LevelingGuidePageDto;
	loading: boolean;
	onResetProgress: () => Promise<void>;
};

function GuideLoadedBody({ page, loading, onResetProgress }: GuideLoadedBodyProps): JSX.Element {
	const actLabel = `Act ${page.position.actIndex + 1}`;
	const progressLabel = formatProgress(page);

	return (
		<div className="guideSnippetProgress">
			<div className="guideSnippetProgressRow">
				<span className="guideSnippetActLabel">{actLabel}</span>
				<span className="guideSnippetPageLabel">{progressLabel}</span>
			</div>
			<div className="guideSnippetProgressBar">
				<div
					className="guideSnippetProgressFill"
					style={{ width: `${(page.position.pageIndex / Math.max(page.pageCountInAct - 1, 1)) * 100}%` }}
				/>
			</div>
			<button
				type="button"
				className="guideSnippetResetButton"
				onClick={() => void onResetProgress()}
				disabled={loading}>
				RESET
			</button>
		</div>
	);
}

export function LevelingGuideDashboardSnippet(props: LevelingGuideDashboardSnippetProps): JSX.Element {
	const { page, loading } = props;
	const isLoaded = page !== null;
	const overlayToggleLabel = props.overlayVisible ? 'HIDE OVERLAY' : 'SHOW OVERLAY';
	const handleOverlayToggle = props.overlayVisible ? props.onHideOverlay : props.onShowOverlay;

	return (
		<ModuleSnippet
			title="Leveling Guide"
			active={isLoaded}
			action={
				isLoaded
					? { type: 'primary', label: overlayToggleLabel, onClick: () => void handleOverlayToggle(), disabled: loading }
					: { type: 'primary', label: 'LOAD DEFAULT GUIDE', onClick: () => void props.onLoadGuide(), disabled: loading }
			}
			onSettingsClick={props.onOpenSettings}>
			{isLoaded && <GuideLoadedBody page={page} loading={loading} onResetProgress={props.onResetProgress} />}
			{props.error !== null && <div className="guideSnippetError">{props.error}</div>}
		</ModuleSnippet>
	);
}
