import { JSX, useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { MainView } from './components/MainView';
import { OverlayPanel } from './components/OverlayPanel';
import { LevelingGuideContent } from './components/LevelingGuideContent';
import type { LevelingGuidePageDto } from './types/guide';

type ViewMode = 'main' | 'overlay_panel';

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
	if (view === 'overlay-panel') {
		return 'overlay_panel';
	}
	return 'main';
}

function App(): JSX.Element {
	const viewMode = useMemo<ViewMode>(() => getViewMode(), []);
	const [currentPage, setCurrentPage] = useState<LevelingGuidePageDto | null>(null);
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | null>(null);

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

	const handleNavigate = useCallback(async (direction: 'previous' | 'next' | 'reset'): Promise<void> => {
		setLoading(true);
		setError(null);
		try {
			const command =
				direction === 'previous'
					? 'leveling_guide_previous_page'
					: direction === 'next'
						? 'leveling_guide_next_page'
						: 'leveling_guide_reset_progress';
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

	useEffect((): void => {
		if (viewMode !== 'overlay_panel') {
			return;
		}
		if (currentPage !== null || loading) {
			return;
		}
		void loadGuide();
	}, [currentPage, loadGuide, loading, viewMode]);

	if (viewMode === 'overlay_panel') {
		return (
			<OverlayPanel>
				<LevelingGuideContent
					page={currentPage}
					loading={loading}
					error={error}
					onNavigate={handleNavigate}
					onLoadGuide={loadGuide}
				/>
			</OverlayPanel>
		);
	}

	return <MainView />;
}

export default App;
