import { JSX } from 'react';
import type { Guide, GuidePage, ConditionalPage } from '../types/guide';
import { requestOverlayFocus } from './OverlayPanel';

function isConditionalPage(page: GuidePage): page is ConditionalPage {
	return typeof page === 'object' && 'condition' in page && 'lines' in page;
}

function extractTextLines(page: GuidePage): string[] {
	if (isConditionalPage(page)) {
		return page.lines;
	}
	return page;
}

interface LevelingGuideContentProps {
	guide: Guide | null;
	currentAct: number;
	currentPage: number;
	loading: boolean;
	error: string | null;
	onNavigate: (actIndex: number, pageIndex: number) => void;
	onLoadGuide: () => Promise<void>;
}

export function LevelingGuideContent({
	guide,
	currentAct,
	currentPage,
	loading,
	error,
	onNavigate,
	onLoadGuide,
}: LevelingGuideContentProps): JSX.Element {
	function handlePrevious(): void {
		if (!guide) return;

		let newPage = currentPage - 1;
		let newAct = currentAct;

		if (newPage < 0) {
			newAct -= 1;
			if (newAct < 0) {
				return;
			}
			newPage = guide[newAct].length - 1;
		}

		onNavigate(newAct, newPage);
		void requestOverlayFocus();
	}

	function handleNext(): void {
		if (!guide) return;

		let newPage = currentPage + 1;
		let newAct = currentAct;

		if (newPage >= guide[currentAct].length) {
			newAct += 1;
			if (newAct >= guide.length) {
				return;
			}
			newPage = 0;
		}

		onNavigate(newAct, newPage);
		void requestOverlayFocus();
	}

	if (!guide) {
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

	const act = guide[currentAct];
	if (!act || !act[currentPage]) {
		return <div className="overlayMessage">No content available.</div>;
	}

	const page: GuidePage = act[currentPage];
	const lines = extractTextLines(page);

	return (
		<div className="guideContent">
			<div className="guideHeader">
				Act {currentAct + 1} - Page {currentPage + 1}/{act.length}
			</div>
			<div className="guideSteps">
				{lines.map((line, index) => {
					const cleanLine = line
						.replace(/\(img:[^)]+\)/g, '')
						.replace(/\(color:[^)]+\)/g, '')
						.replace(/\(hint\)__/g, '→')
						.replace(/\(hint\)_/g, '→')
						.replace(/\(quest:[^)]+\)/g, '')
						.replace(/<[^>]+>/g, '')
						.replace(/areaid[^\s;]+/g, '')
						.replace(/;;/g, '-')
						.trim();

					if (!cleanLine) return null;

					return (
						<div key={index} className="guideStep">
							{cleanLine}
						</div>
					);
				})}
			</div>
			<div className="guideNavigation">
				<button type="button" onClick={handlePrevious} disabled={currentAct === 0 && currentPage === 0}>
					← Previous
				</button>
				<button
					type="button"
					onClick={handleNext}
					disabled={currentAct === guide.length - 1 && currentPage === act.length - 1}>
					Next →
				</button>
			</div>
		</div>
	);
}
