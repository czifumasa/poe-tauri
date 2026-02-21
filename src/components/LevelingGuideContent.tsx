import { Fragment, JSX } from 'react';
import type { LevelingGuidePageDto, LevelingGuideSpanDto } from '../types/guide';
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

export function LevelingGuideContent({
	page,
	loading,
	error,
	...rest
}: LevelingGuideContentProps): JSX.Element {
	if (!page) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{error && <div className="overlayError">{error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
				{rest.variant === 'dashboard' && (
					<div className="guideDashboardControls">
						<button
							type="button"
							className="loadGuideButton"
							onClick={() => void rest.onLoadGuide()}
							disabled={loading}>
							Load Guide
						</button>
						<button type="button" onClick={() => void rest.onResetProgress()} disabled>
							Reset
						</button>
					</div>
				)}
			</div>
		);
	}

	const header = `Act ${page.position.actIndex + 1} - Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;

	const handlePrevious = (): void => {
		if (rest.variant !== 'overlay') {
			return;
		}
		void rest.onNavigate('previous');
		void requestOverlayFocus();
	};

	const handleNext = (): void => {
		if (rest.variant !== 'overlay') {
			return;
		}
		void rest.onNavigate('next');
		void requestOverlayFocus();
	};

	const handleReset = (): void => {
		if (rest.variant !== 'dashboard') {
			return;
		}
		void rest.onResetProgress();
	};

	return (
		<div className={rest.variant === 'dashboard' ? 'guideContent guideContentCompact' : 'guideContent'}>
			<div className="guideHeader">{header}</div>
			{rest.variant === 'overlay' && (
				<div className="guideSteps">
					{page.lines.map((line, lineIndex) => (
						<div
							key={lineIndex}
							className={line.isHint ? 'guideStep guideStepHint' : 'guideStep'}>
							{line.spans.map((span, spanIndex) => renderSpan(span, `${lineIndex}-${spanIndex}`))}
						</div>
					))}
				</div>
			)}
			<div className="guideNavigation">
				{rest.variant === 'overlay' ? (
					<>
						<button type="button" onClick={handlePrevious} disabled={!page.hasPrevious || loading}>
							← Previous
						</button>
						<button type="button" onClick={handleNext} disabled={!page.hasNext || loading}>
							Next →
						</button>
					</>
				) : (
					<>
						<button type="button" onClick={() => void rest.onLoadGuide()} disabled={loading}>
							Load
						</button>
						<button type="button" onClick={handleReset} disabled={loading}>
							Reset
						</button>
					</>
				)}
			</div>
		</div>
	);
}
