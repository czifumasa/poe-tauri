import { Fragment, JSX } from 'react';
import type { LevelingGuideLineDto, LevelingGuidePageDto, LevelingGuideSpanDto } from '../types/guide';
import { requestOverlayFocus } from './OverlayPanel';

type OverlayLevelingGuideContentProps = {
	variant: 'overlay';
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onNavigate: (direction: 'previous' | 'next') => Promise<void>;
};

type DashboardLevelingGuideContentProps = {
	variant: 'dashboard';
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
};

type LevelingGuideContentProps = OverlayLevelingGuideContentProps | DashboardLevelingGuideContentProps;

function renderSpan(span: LevelingGuideSpanDto, key: string): JSX.Element {
	if (span.type === 'image') {
		return <img key={key} className="guideInlineImage" src={span.dataUri} alt={span.key} />;
	}
	return <Fragment key={key}>{span.text}</Fragment>;
}

function renderLine(line: LevelingGuideLineDto, lineIndex: number): JSX.Element {
	const lineClassName = line.isHint ? 'guideStep guideStepHint' : 'guideStep';
	return (
		<div key={lineIndex} className={lineClassName}>
			{line.spans.map((span, spanIndex) => renderSpan(span, `${lineIndex}-${spanIndex}`))}
		</div>
	);
}

function getActLabel(page: LevelingGuidePageDto): string {
	return `ACT ${page.position.actIndex + 1}`;
}

function getPageCounterLabel(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

function getDashboardHeaderLabel(page: LevelingGuidePageDto): string {
	const actLabel = `Act ${page.position.actIndex + 1}`;
	return `${actLabel} - Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;
}

function OverlayGuideContent(props: OverlayLevelingGuideContentProps): JSX.Element {
	const { page, loading, onNavigate } = props;
	if (page === null) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{props.error && <div className="overlayError">{props.error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
			</div>
		);
	}

	const handlePrevious = (): void => {
		void onNavigate('previous');
		void requestOverlayFocus();
	};

	const handleNext = (): void => {
		void onNavigate('next');
		void requestOverlayFocus();
	};

	return (
		<div className="guideContent guideContentOverlay">
			<div className="guideHeader guideHeaderOverlay">
				<div className="guideHeaderLeft">{getActLabel(page)}</div>
				<div className="guideHeaderRight">{getPageCounterLabel(page)}</div>
			</div>
			<div className="guideSteps">{page.lines.map((line, lineIndex) => renderLine(line, lineIndex))}</div>
			<div className="guideNavigation">
				<button
					type="button"
					className="guideNavButton"
					onClick={handlePrevious}
					disabled={!page.hasPrevious || loading}>
					{'← PREV'}
				</button>
				<button type="button" className="guideNavButton" onClick={handleNext} disabled={!page.hasNext || loading}>
					{'NEXT →'}
				</button>
			</div>
		</div>
	);
}

function DashboardGuideContent(props: DashboardLevelingGuideContentProps): JSX.Element {
	const { page, loading } = props;
	if (page === null) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{props.error && <div className="overlayError">{props.error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
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

export function LevelingGuideContent({ page, loading, error, ...rest }: LevelingGuideContentProps): JSX.Element {
	if (rest.variant === 'overlay') {
		return (
			<OverlayGuideContent variant="overlay" page={page} loading={loading} error={error} onNavigate={rest.onNavigate} />
		);
	}

	return (
		<DashboardGuideContent
			variant="dashboard"
			page={page}
			loading={loading}
			error={error}
			onLoadGuide={rest.onLoadGuide}
			onResetProgress={rest.onResetProgress}
		/>
	);
}
