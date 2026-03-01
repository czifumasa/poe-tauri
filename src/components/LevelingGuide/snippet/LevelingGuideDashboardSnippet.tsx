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

const DESCRIPTION = 'Step-by-step act progression with gem and quest tracking.';

function formatProgress(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

function GuideLoadedBody({ page }: { page: LevelingGuidePageDto }): JSX.Element {
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
		</div>
	);
}

export function LevelingGuideDashboardSnippet(props: LevelingGuideDashboardSnippetProps): JSX.Element {
	const { page, loading } = props;
	const isLoaded = page !== null;
	const overlayToggleLabel = props.overlayVisible ? 'HIDE' : 'SHOW';
	const handleOverlayToggle = props.overlayVisible ? props.onHideOverlay : props.onShowOverlay;

	return (
		<ModuleSnippet
			title="Leveling Guide"
			description={DESCRIPTION}
			active={isLoaded}
			showButton={isLoaded ? { label: overlayToggleLabel, onClick: () => void handleOverlayToggle(), disabled: loading } : undefined}
			action={
				isLoaded
					? { type: 'primary', label: 'RESET', onClick: () => void props.onResetProgress(), disabled: loading }
					: { type: 'primary', label: 'LOAD DEFAULT GUIDE', onClick: () => void props.onLoadGuide(), disabled: loading }
			}
			onSettingsClick={props.onOpenSettings}>
			{isLoaded && <GuideLoadedBody page={page} />}
			{props.error !== null && <div className="guideSnippetError">{props.error}</div>}
		</ModuleSnippet>
	);
}
