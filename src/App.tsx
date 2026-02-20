import { JSX, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { MainView } from './components/MainView';
import { OverlayPanel } from './components/OverlayPanel';
import { LevelingGuideContent } from './components/LevelingGuideContent';
import type { Guide } from './types/guide';

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
	const [guide, setGuide] = useState<Guide | null>(null);
	const [currentAct, setCurrentAct] = useState<number>(0);
	const [currentPage, setCurrentPage] = useState<number>(0);
	const [loading, setLoading] = useState<boolean>(false);
	const [error, setError] = useState<string | null>(null);

	async function loadGuide(): Promise<void> {
		setLoading(true);
		setError(null);
		try {
			const guideData = await invoke<Guide>('load_guide');
			setGuide(guideData);
			setCurrentAct(0);
			setCurrentPage(0);
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			setError(`Failed to load guide: ${errorMessage}`);
			console.error('Failed to load guide:', err);
		} finally {
			setLoading(false);
		}
	}

	function handleNavigate(actIndex: number, pageIndex: number): void {
		setCurrentAct(actIndex);
		setCurrentPage(pageIndex);
	}

	if (viewMode === 'overlay_panel') {
		return (
			<OverlayPanel>
				<LevelingGuideContent
					guide={guide}
					currentAct={currentAct}
					currentPage={currentPage}
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
