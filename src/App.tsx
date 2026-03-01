import { JSX, useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import './App.css';
import { MainView } from './components/MainView/MainView';
import { OverlayPanel } from './components/OverlayPanel/OverlayPanel';
import { ModuleSnippet } from './components/ModuleSnippet/ModuleSnippet.tsx';
import type { LevelingGuidePageDto } from './types/Guide.ts';
import type { BanditsChoice, LevelingGuideSettings } from './types/Settings.ts';
import { HINT_TOOLTIP_VIEW_QUERY_VALUE, OVERLAY_VIEW_QUERY_VALUE } from './constants/WindowIdentifiers.ts';
import { LevelingGuideOverlay } from './components/LevelingGuide/overlay/LevelingGuideOverlay.tsx';
import { LevelingGuideDashboardSnippet } from './components/LevelingGuide/snippet/LevelingGuideDashboardSnippet.tsx';
import { LevelingGuideSettingsPanel } from './components/LevelingGuide/settings/LevelingGuideSettingsPanel.tsx';
import { HintTooltipView } from './components/HintTooltip/HintTooltipView.tsx';

type ViewMode = 'main' | 'overlay' | 'hintTooltip';

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
	if (view === HINT_TOOLTIP_VIEW_QUERY_VALUE) {
		return 'hintTooltip';
	}
	return 'main';
}

type OverlayHeightVariant = 'oneLine' | 'twoLines' | 'threeLines' | 'fourLines' | 'fiveLines';

function clampOverlayVariantLineCount(lineCount: number): number {
	if (lineCount <= 1) {
		return 1;
	}
	if (lineCount >= 6) {
		return 6;
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
	const headerHeightPx = 36;
	const footerHeightPx = 36;
	const lineHeightPx = 24;
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
	const [settingsOpen, setSettingsOpen] = useState<boolean>(false);
	const [overlayVisible, setOverlayVisible] = useState<boolean>(false);
	const [settings, setSettings] = useState<LevelingGuideSettings>({
		leagueStart: true,
		overlayPosition: null,
		optionalQuests: true,
		levelRecommendations: true,
		banditsChoice: 'KillAll',
		clientLogPath: null,
		gemsEnabled: false,
		pobCode: null,
	});
	const [settingsLoading, setSettingsLoading] = useState<boolean>(true);
	const [pobClass, setPobClass] = useState<string | null>(null);
	const [pobGemCount, setPobGemCount] = useState<number | null>(null);

	useEffect((): (() => void) => {
		let isDisposed = false;
		setSettingsLoading(true);
		void (async (): Promise<void> => {
			try {
				const persistedSettings = await invoke<LevelingGuideSettings>('settings_get_leveling_guide');
				if (isDisposed) {
					return;
				}
				setSettings(persistedSettings);
			} catch (err) {
				console.error('Failed to initialize leveling guide settings:', err);
			} finally {
				if (!isDisposed) {
					setSettingsLoading(false);
				}
			}
		})();
		return (): void => {
			isDisposed = true;
		};
	}, []);

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
				} else {
					const loadedPage = await invoke<LevelingGuidePageDto>('load_guide');
					if (isDisposed) {
						return;
					}
					setCurrentPage(loadedPage);
				}

				const status = await invoke<{ class: string; gemNames: string[] } | null>('leveling_guide_get_pob_status');
				if (!isDisposed && status !== null) {
					setPobClass(status.class);
					setPobGemCount(status.gemNames.length);
				}
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

	const updateLeagueStart = useCallback(
		async (nextValue: boolean): Promise<void> => {
			const updatedSettings: LevelingGuideSettings = { ...settings, leagueStart: nextValue };
			setSettings(updatedSettings);
			try {
				await invoke('settings_set_leveling_guide', { settings: updatedSettings });
			} catch (err) {
				console.error('Failed to persist leveling guide settings:', err);
			}
		},
		[settings],
	);

	const updateOptionalQuests = useCallback(
		async (nextValue: boolean): Promise<void> => {
			const updatedSettings: LevelingGuideSettings = { ...settings, optionalQuests: nextValue };
			setSettings(updatedSettings);
			try {
				await invoke('settings_set_leveling_guide', { settings: updatedSettings });
			} catch (err) {
				console.error('Failed to persist leveling guide settings:', err);
			}
		},
		[settings],
	);

	const updateLevelRecommendations = useCallback(
		async (nextValue: boolean): Promise<void> => {
			const updatedSettings: LevelingGuideSettings = { ...settings, levelRecommendations: nextValue };
			setSettings(updatedSettings);
			try {
				await invoke('settings_set_leveling_guide', { settings: updatedSettings });
			} catch (err) {
				console.error('Failed to persist leveling guide settings:', err);
			}
		},
		[settings],
	);

	const updateBanditsChoice = useCallback(
		async (nextValue: BanditsChoice): Promise<void> => {
			const updatedSettings: LevelingGuideSettings = { ...settings, banditsChoice: nextValue };
			setSettings(updatedSettings);
			try {
				await invoke('settings_set_leveling_guide', { settings: updatedSettings });
			} catch (err) {
				console.error('Failed to persist leveling guide settings:', err);
			}
		},
		[settings],
	);

	const browseClientLogPath = useCallback(async (): Promise<void> => {
		const selected = await open({
			title: 'Select Client.txt',
			multiple: false,
			directory: false,
			filters: [{ name: 'Log files', extensions: ['txt'] }],
		});
		if (selected === null) {
			return;
		}
		const updatedSettings: LevelingGuideSettings = { ...settings, clientLogPath: selected };
		setSettings(updatedSettings);
		try {
			await invoke('settings_set_leveling_guide', { settings: updatedSettings });
		} catch (err) {
			console.error('Failed to persist client log path:', err);
		}
	}, [settings]);

	const clearClientLogPath = useCallback(async (): Promise<void> => {
		const updatedSettings: LevelingGuideSettings = { ...settings, clientLogPath: null };
		setSettings(updatedSettings);
		try {
			await invoke('settings_set_leveling_guide', { settings: updatedSettings });
		} catch (err) {
			console.error('Failed to clear client log path:', err);
		}
	}, [settings]);

	const updateGemsEnabled = useCallback(
		async (nextValue: boolean): Promise<void> => {
			const updatedSettings: LevelingGuideSettings = { ...settings, gemsEnabled: nextValue };
			setSettings(updatedSettings);
			try {
				await invoke('settings_set_leveling_guide', { settings: updatedSettings });
				const page = await invoke<LevelingGuidePageDto>('leveling_guide_reapply_gems');
				setCurrentPage(page);
			} catch (err) {
				console.error('Failed to toggle gems setting:', err);
			}
		},
		[settings],
	);

	const importPob = useCallback(async (pobCode: string): Promise<void> => {
		try {
			const page = await invoke<LevelingGuidePageDto>('leveling_guide_import_pob', { pobCode });
			setCurrentPage(page);
			const status = await invoke<{ class: string; gemNames: string[] } | null>('leveling_guide_get_pob_status');
			if (status !== null) {
				setPobClass(status.class);
				setPobGemCount(status.gemNames.length);
			}
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			throw new Error(errorMessage);
		}
	}, []);

	const showOverlay = useCallback(async (): Promise<void> => {
		await invoke('show_overlay');
		setOverlayVisible(true);
	}, []);

	const hideOverlay = useCallback(async (): Promise<void> => {
		await invoke('hide_overlay');
		setOverlayVisible(false);
	}, []);

	useEffect((): (() => void) => {
		let isDisposed = false;
		void (async (): Promise<void> => {
			try {
				const visible = await invoke<boolean>('overlay_is_visible');
				if (!isDisposed) {
					setOverlayVisible(visible);
				}
			} catch (err) {
				console.error('Failed to query overlay visibility:', err);
			}
		})();
		return (): void => {
			isDisposed = true;
		};
	}, []);

	const openLevelingGuideSettings = useCallback((): void => {
		setSettingsOpen(true);
	}, []);

	const closeLevelingGuideSettings = useCallback((): void => {
		setSettingsOpen(false);
	}, []);

	if (viewMode === 'overlay') {
		const overlaySize = getOverlayLogicalSize(currentPage);
		return (
			<OverlayPanel logicalWidthPx={overlaySize.widthPx} logicalHeightPx={overlaySize.heightPx}>
				<LevelingGuideOverlay page={currentPage} loading={loading} error={error} onNavigate={handleNavigate} />
			</OverlayPanel>
		);
	}

	if (viewMode === 'hintTooltip') {
		return <HintTooltipView />;
	}

	const settingsContent = settingsOpen ? (
		<LevelingGuideSettingsPanel
			settingsLoading={settingsLoading}
			leagueStart={settings.leagueStart}
			onLeagueStartChange={updateLeagueStart}
			optionalQuests={settings.optionalQuests}
			onOptionalQuestsChange={updateOptionalQuests}
			levelRecommendations={settings.levelRecommendations}
			onLevelRecommendationsChange={updateLevelRecommendations}
			banditsChoice={settings.banditsChoice}
			onBanditsChoiceChange={updateBanditsChoice}
			clientLogPath={settings.clientLogPath}
			onClientLogPathBrowse={browseClientLogPath}
			onClientLogPathClear={clearClientLogPath}
			gemsEnabled={settings.gemsEnabled}
			onGemsEnabledChange={updateGemsEnabled}
			onImportPob={importPob}
			pobClass={pobClass}
			pobGemCount={pobGemCount}
			onBack={closeLevelingGuideSettings}
		/>
	) : undefined;

	return (
		<MainView
			settingsContent={settingsContent}
			overlaysVisible={overlayVisible}
			onShowAllOverlays={showOverlay}
			onHideAllOverlays={hideOverlay}>
			<LevelingGuideDashboardSnippet
				page={currentPage}
				overlayVisible={overlayVisible}
				loading={loading}
				error={error}
				onLoadGuide={loadGuide}
				onResetProgress={resetProgress}
				onShowOverlay={showOverlay}
				onHideOverlay={hideOverlay}
				onOpenSettings={openLevelingGuideSettings}
			/>
			<ModuleSnippet
				title="Map Tracking"
				description="Tracks completed maps and their content."
				disabled
				action={{ type: 'comingSoon' }}
				onSettingsClick={() => {}}
				settingsDisabled
			/>
			<ModuleSnippet
				title="Build Planner"
				description="Skill tree overlay with passive node suggestions"
				disabled
				action={{ type: 'comingSoon' }}
				onSettingsClick={() => {}}
				settingsDisabled
			/>
		</MainView>
	);
}

export default App;
