import { JSX } from 'react';
import type { LevelingGuidePageDto } from '../types/guide';
import { requestOverlayFocus } from './OverlayPanel';

interface LevelingGuideContentProps {
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onNavigate: (direction: 'previous' | 'next' | 'reset') => Promise<void>;
	onLoadGuide: () => Promise<void>;
}

export function LevelingGuideContent({
	page,
	loading,
	error,
	onNavigate,
	onLoadGuide,
}: LevelingGuideContentProps): JSX.Element {
	function handlePrevious(): void {
		if (!page) return;
		void onNavigate('previous');
		void requestOverlayFocus();
	}

	function handleNext(): void {
		if (!page) return;
		void onNavigate('next');
		void requestOverlayFocus();
	}

	function handleReset(): void {
		if (!page) return;
		void onNavigate('reset');
		void requestOverlayFocus();
	}

	if (!page) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide not loaded.</div>
				{error && <div className="overlayError">{error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
				<button
					type="button"
					className="loadGuideButton"
					onClick={() => {
						void onLoadGuide();
						void requestOverlayFocus();
					}}
					disabled={loading}>
					Load Guide
				</button>
			</div>
		);
	}

	const header = `Act ${page.position.actIndex + 1} - Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;

	return (
		<div className="guideContent">
			<div className="guideHeader">{header}</div>
			<div className="guideSteps">
				{page.lines.map((line, index) => (
					<div key={index} className="guideStep">
						{line}
					</div>
				))}
			</div>
			<div className="guideNavigation">
				<button type="button" onClick={handlePrevious} disabled={!page.hasPrevious || loading}>
					← Previous
				</button>
				<button type="button" onClick={handleReset} disabled={loading}>
					Reset
				</button>
				<button
					type="button"
					onClick={handleNext}
					disabled={!page.hasNext || loading}>
					Next →
				</button>
			</div>
		</div>
	);
}
