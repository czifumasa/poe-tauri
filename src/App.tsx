import { JSX, useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import './App.css';
import { MainView } from './components/MainView/MainView';
import { OverlayPanel } from './components/OverlayPanel/OverlayPanel';
import type { LevelingGuidePageDto } from './types/Guide.ts';
import { OVERLAY_VIEW_QUERY_VALUE } from './constants/WindowIdentifiers.ts';
import { LevelingGuideOverlay } from './components/LevelingGuide/overlay/LevelingGuideOverlay.tsx';
import { LevelingGuideDashboardSnippet } from './components/LevelingGuide/snippet/LevelingGuideDashboardSnippet.tsx';

type ViewMode = 'main' | 'overlay';

function formatInvokeError(error: unknown): string {
	if (error instanceof Error) {
		return error.message;
	}

	if (typeof error === 'string') {
		return error;
	}

	if (typeof error === 'object' && error !== null) {
		const message = (error as { message?: unknown }).message;
		if (typeof message === 'string') {
			return message;
		}
		try {
			return JSON.stringify(error);
		} catch {
			return String(error);
		}
	}

	return String(error);
}

function getViewMode(): ViewMode {
	const params = new URLSearchParams(window.location.search);
	const view = params.get('view');
	if (view === OVERLAY_VIEW_QUERY_VALUE) {
		return 'overlay';
	}
	return 'main';
}

type OverlayHeightVariant = 'oneLine' | 'twoLines' | 'threeLines' | 'fourLines' | 'fiveLines';

function clampOverlayVariantLineCount(lineCount: number): number {
	if (lineCount <= 1) {
		return 1;
	}
	if (lineCount >= 5) {
		return 5;
	}
	return lineCount;
}

function getOverlayHeightVariant(lineCount: number): OverlayHeightVariant {
	switch (lineCount) {
		case 1:
			return 'oneLine';
		case 2:
			return 'twoLines';
		case 3:
			return 'threeLines';
		case 4:
			return 'fourLines';
		default:
			return 'fiveLines';
	}
}

function getOverlayLogicalSize(page: LevelingGuidePageDto | null): {
	widthPx: number;
	heightPx: number;
	variant: OverlayHeightVariant;
} {
	const headerHeightPx = 50;
	const footerHeightPx = 50;
	const lineHeightPx = 30;
	const widthPx = 340;
	const rawLineCount = page?.lines.length ?? 1;
	const variantLineCount = clampOverlayVariantLineCount(rawLineCount);
	const variant = getOverlayHeightVariant(variantLineCount);
	const heightPx = headerHeightPx + footerHeightPx + lineHeightPx * variantLineCount;
	return { widthPx, heightPx, variant };
}

function App(): JSX.Element {
	const viewMode = useMemo<ViewMode>(() => getViewMode(), []);
	const [currentPage, setCurrentPage] = useState<LevelingGuidePageDto | null>(null);
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | null>(null);

	useEffect((): (() => void) => {
		let isDisposed = false;

		void (async (): Promise<void> => {
			setLoading(true);
			setError(null);
			try {
				const existingPage = await invoke<LevelingGuidePageDto | null>('leveling_guide_get_current_page');
				if (isDisposed) {
					return;
				}

				if (existingPage !== null) {
					setCurrentPage(existingPage);
					return;
				}

				const loadedPage = await invoke<LevelingGuidePageDto>('load_guide');
				if (isDisposed) {
					return;
				}
				setCurrentPage(loadedPage);
			} catch (err) {
				if (isDisposed) {
					return;
				}
				const errorMessage = formatInvokeError(err);
				setError(`Failed to initialize guide: ${errorMessage}`);
				console.error('Failed to initialize guide:', err);
			} finally {
				if (!isDisposed) {
					setLoading(false);
				}
			}
		})();

		return (): void => {
			isDisposed = true;
		};
	}, []);

	const loadGuide = useCallback(async (): Promise<void> => {
		setLoading(true);
		setError(null);
		try {
			const page = await invoke<LevelingGuidePageDto>('load_guide');
			setCurrentPage(page);
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			setError(`Failed to load guide: ${errorMessage}`);
			console.error('Failed to load guide:', err);
		} finally {
			setLoading(false);
		}
	}, []);

	const handleNavigate = useCallback(async (direction: 'previous' | 'next'): Promise<void> => {
		setLoading(true);
		setError(null);
		try {
			const command = direction === 'previous' ? 'leveling_guide_previous_page' : 'leveling_guide_next_page';
			const page = await invoke<LevelingGuidePageDto>(command);
			setCurrentPage(page);
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			setError(`Failed to navigate guide: ${errorMessage}`);
			console.error('Failed to navigate guide:', err);
		} finally {
			setLoading(false);
		}
	}, []);

	const resetProgress = useCallback(async (): Promise<void> => {
		if (currentPage === null) {
			return;
		}

		setLoading(true);
		setError(null);
		try {
			const page = await invoke<LevelingGuidePageDto>('leveling_guide_reset_progress');
			setCurrentPage(page);
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			setError(`Failed to reset guide: ${errorMessage}`);
			console.error('Failed to reset guide:', err);
		} finally {
			setLoading(false);
		}
	}, [currentPage]);

	useEffect((): (() => void) => {
		let isDisposed = false;
		let unlisten: (() => void) | null = null;

		void (async (): Promise<void> => {
			try {
				unlisten = await listen<LevelingGuidePageDto>('leveling_guide_page_updated', (event) => {
					if (isDisposed) {
						return;
					}
					setCurrentPage(event.payload);
				});
			} catch (err) {
				console.error('Failed to listen for leveling guide updates:', err);
			}
		})();

		return (): void => {
			isDisposed = true;
			if (unlisten !== null) {
				unlisten();
			}
		};
	}, []);

	if (viewMode === 'overlay') {
		const overlaySize = getOverlayLogicalSize(currentPage);
		return (
			<OverlayPanel logicalWidthPx={overlaySize.widthPx} logicalHeightPx={overlaySize.heightPx}>
				<LevelingGuideOverlay page={currentPage} loading={loading} error={error} onNavigate={handleNavigate} />
			</OverlayPanel>
		);
	}

	return (
		<MainView>
			<LevelingGuideDashboardSnippet
				page={currentPage}
				loading={loading}
				error={error}
				onLoadGuide={loadGuide}
				onResetProgress={resetProgress}
			/>
		</MainView>
	);
}

export default App;
