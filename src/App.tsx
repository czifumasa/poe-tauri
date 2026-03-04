import { JSX, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';
import { open } from '@tauri-apps/plugin-dialog';
import './App.css';
import { MainView } from './components/MainView/MainView';
import { OverlayPanel } from './components/OverlayPanel/OverlayPanel';
import { ModuleSnippet } from './components/ModuleSnippet/ModuleSnippet.tsx';
import type { LevelingGuidePageDto } from './types/Guide.ts';
import type { BanditsChoice, LevelingGuideSettings, PobSettings } from './types/Settings.ts';
import type { TimerSettings, TimerState } from './types/Timer.ts';
import { HINT_TOOLTIP_VIEW_QUERY_VALUE, OVERLAY_VIEW_QUERY_VALUE } from './constants/WindowIdentifiers.ts';
import { LevelingGuideOverlay } from './components/LevelingGuide/overlay/LevelingGuideOverlay.tsx';
import { LevelingGuideDashboardSnippet } from './components/LevelingGuide/snippet/LevelingGuideDashboardSnippet.tsx';
import { LevelingGuideSettingsPanel } from './components/LevelingGuide/settings/LevelingGuideSettingsPanel.tsx';
import { HintTooltipView } from './components/HintTooltip/HintTooltipView.tsx';
import { PobImportDashboardSnippet } from './components/PobImport/snippet/PobImportDashboardSnippet.tsx';
import { PobImportSettingsPanel } from './components/PobImport/settings/PobImportSettingsPanel.tsx';
import { TimerDashboardSnippet } from './components/Timer/snippet/TimerDashboardSnippet.tsx';
import { TimerSettingsPanel } from './components/Timer/settings/TimerSettingsPanel.tsx';
import { SettingsPage, type SettingsTab } from './components/SettingsPage/SettingsPage.tsx';

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
	const headerHeightPx = 52;
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
	const [appVersion, setAppVersion] = useState<string | null>(null);
	const [activeSettingsTab, setActiveSettingsTab] = useState<SettingsTab | null>(null);
	const [overlayVisible, setOverlayVisible] = useState<boolean>(false);
	const [settings, setSettings] = useState<LevelingGuideSettings>({
		leagueStart: true,
		overlayPosition: null,
		optionalQuests: true,
		levelRecommendations: true,
		banditsChoice: 'KillAll',
		clientLogPath: null,
		gemsEnabled: false,
	});
	const [settingsLoading, setSettingsLoading] = useState<boolean>(true);
	const [pobSettings, setPobSettings] = useState<PobSettings>({ slots: [], currentSlotIndex: null });
	const [pobSettingsLoading, setPobSettingsLoading] = useState<boolean>(true);
	const [timerSettings, setTimerSettings] = useState<TimerSettings>({
		actTimerEnabled: false,
		campaignTimerEnabled: false,
	});
	const [timerState, setTimerState] = useState<TimerState>({
		status: 'idle',
		currentActIndex: 0,
		actElapsedMs: Array.from({ length: 10 }, () => 0),
		currentActElapsedMs: 0,
		campaignElapsedMs: 0,
	});
	const timerTickRef = useRef<number | null>(null);
	const timerStateRef = useRef<TimerState>(timerState);
	timerStateRef.current = timerState;
	const [resetEpoch, setResetEpoch] = useState<number>(0);

	useEffect((): (() => void) => {
		let isDisposed = false;
		void (async (): Promise<void> => {
			try {
				const version = await getVersion();
				if (!isDisposed) {
					setAppVersion(`v${version}`);
				}
			} catch (err) {
				console.error('Failed to read app version:', err);
				if (!isDisposed) {
					setAppVersion(null);
				}
			}
		})();
		return (): void => {
			isDisposed = true;
		};
	}, []);

	useEffect((): (() => void) => {
		let isDisposed = false;
		setSettingsLoading(true);
		setPobSettingsLoading(true);
		void (async (): Promise<void> => {
			try {
				const [persistedSettings, persistedPobSettings, persistedTimerSettings, persistedTimerState] =
					await Promise.all([
						invoke<LevelingGuideSettings>('settings_get_leveling_guide'),
						invoke<PobSettings>('pob_settings_get'),
						invoke<TimerSettings>('timer_get_settings'),
						invoke<TimerState>('timer_load_state'),
					]);
				if (isDisposed) {
					return;
				}
				setSettings(persistedSettings);
				setPobSettings(persistedPobSettings);
				setTimerSettings(persistedTimerSettings);
				setTimerState(persistedTimerState);
			} catch (err) {
				console.error('Failed to initialize settings:', err);
			} finally {
				if (!isDisposed) {
					setSettingsLoading(false);
					setPobSettingsLoading(false);
				}
			}
		})();
		return (): void => {
			isDisposed = true;
		};
	}, [resetEpoch]);

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

				const hasPersistedProgress = await invoke<boolean>('leveling_guide_has_persisted_progress');
				if (isDisposed) {
					return;
				}

				if (hasPersistedProgress) {
					const restoredPage = await invoke<LevelingGuidePageDto>('load_guide');
					if (isDisposed) {
						return;
					}
					setCurrentPage(restoredPage);
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
	}, [resetEpoch]);

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
		let unlistenPageUpdated: (() => void) | null = null;
		let unlistenCleared: (() => void) | null = null;
		let unlistenTimerState: (() => void) | null = null;

		void (async (): Promise<void> => {
			try {
				const [pageUpdatedUnsub, clearedUnsub, timerStateUnsub] = await Promise.all([
					listen<LevelingGuidePageDto>('leveling_guide_page_updated', (event) => {
						if (isDisposed) {
							return;
						}
						setCurrentPage(event.payload);
					}),
					listen<void>('leveling_guide_cleared', () => {
						if (isDisposed) {
							return;
						}
						setCurrentPage(null);
					}),
					listen<TimerState>('timer_state_updated', (event) => {
						if (isDisposed) {
							return;
						}
						setTimerState(event.payload);
					}),
				]);
				unlistenPageUpdated = pageUpdatedUnsub;
				unlistenCleared = clearedUnsub;
				unlistenTimerState = timerStateUnsub;
			} catch (err) {
				console.error('Failed to listen for leveling guide updates:', err);
			}
		})();

		return (): void => {
			isDisposed = true;
			if (unlistenPageUpdated !== null) {
				unlistenPageUpdated();
			}
			if (unlistenCleared !== null) {
				unlistenCleared();
			}
			if (unlistenTimerState !== null) {
				unlistenTimerState();
			}
		};
	}, []);

	useEffect((): (() => void) => {
		if (timerState.status !== 'running') {
			if (timerTickRef.current !== null) {
				window.clearInterval(timerTickRef.current);
				timerTickRef.current = null;
			}
			return (): void => {};
		}

		const startedAt = Date.now();
		const baseActMs = timerState.currentActElapsedMs;
		const baseCampaignMs = timerState.campaignElapsedMs;

		timerTickRef.current = window.setInterval((): void => {
			const elapsed = Date.now() - startedAt;
			setTimerState((prev) => ({
				...prev,
				currentActElapsedMs: baseActMs + elapsed,
				campaignElapsedMs: baseCampaignMs + elapsed,
			}));
		}, 1000);

		return (): void => {
			if (timerTickRef.current !== null) {
				window.clearInterval(timerTickRef.current);
				timerTickRef.current = null;
			}
		};
		// eslint-disable-next-line react-hooks/exhaustive-deps
	}, [timerState.status]);

	const handleTimerAction = useCallback((action: 'start' | 'pause' | 'resume' | 'reset'): void => {
		const commandMap = {
			start: 'timer_start',
			pause: 'timer_pause',
			resume: 'timer_resume',
			reset: 'timer_reset',
		} as const;
		void invoke<TimerState>(commandMap[action])
			.then((state) => {
				setTimerState(state);
			})
			.catch((err: unknown) => {
				console.error(`Failed to ${action} timer:`, err);
			});
	}, []);

	const updateTimerActEnabled = useCallback(
		(nextValue: boolean): void => {
			const updated: TimerSettings = { ...timerSettings, actTimerEnabled: nextValue };
			setTimerSettings(updated);
			void invoke('timer_set_settings', {
				actTimerEnabled: updated.actTimerEnabled,
				campaignTimerEnabled: updated.campaignTimerEnabled,
			}).catch((err: unknown) => {
				console.error('Failed to persist timer settings:', err);
			});
		},
		[timerSettings],
	);

	const updateTimerCampaignEnabled = useCallback(
		(nextValue: boolean): void => {
			const updated: TimerSettings = { ...timerSettings, campaignTimerEnabled: nextValue };
			setTimerSettings(updated);
			void invoke('timer_set_settings', {
				actTimerEnabled: updated.actTimerEnabled,
				campaignTimerEnabled: updated.campaignTimerEnabled,
			}).catch((err: unknown) => {
				console.error('Failed to persist timer settings:', err);
			});
		},
		[timerSettings],
	);

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

	const addPobSlot = useCallback(async (pobCode: string): Promise<void> => {
		try {
			const updated = await invoke<PobSettings>('pob_settings_add_slot', { pobCode });
			setPobSettings(updated);
		} catch (err) {
			const errorMessage = formatInvokeError(err);
			throw new Error(errorMessage);
		}
	}, []);

	const removePobSlot = useCallback(async (slotIndex: number): Promise<void> => {
		try {
			const updated = await invoke<PobSettings>('pob_settings_remove_slot', { slotIndex });
			setPobSettings(updated);
		} catch (err) {
			console.error('Failed to remove PoB slot:', err);
		}
	}, []);

	const setCurrentPobSlot = useCallback(async (slotIndex: number): Promise<void> => {
		try {
			const updated = await invoke<PobSettings>('pob_settings_set_current_slot', { slotIndex });
			setPobSettings(updated);
		} catch (err) {
			console.error('Failed to set current PoB slot:', err);
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
	}, [resetEpoch]);

	const wipeSettings = useCallback(async (): Promise<void> => {
		try {
			if (overlayVisible) {
				await invoke('hide_overlay');
			}
			await invoke('overlay_reset_to_default_position');
			await invoke('settings_wipe');
			setCurrentPage(null);
			setError(null);
			setActiveSettingsTab(null);
			setOverlayVisible(false);
			setSettings({
				leagueStart: true,
				overlayPosition: null,
				optionalQuests: true,
				levelRecommendations: true,
				banditsChoice: 'KillAll',
				clientLogPath: null,
				gemsEnabled: false,
			});
			setPobSettings({ slots: [], currentSlotIndex: null });
			setTimerSettings({ actTimerEnabled: false, campaignTimerEnabled: false });
			setTimerState({
				status: 'idle',
				currentActIndex: 0,
				actElapsedMs: Array.from({ length: 10 }, () => 0),
				currentActElapsedMs: 0,
				campaignElapsedMs: 0,
			});
			setResetEpoch((prev) => prev + 1);
		} catch (err) {
			console.error('Failed to wipe settings:', err);
		}
	}, [overlayVisible]);

	const openSettings = useCallback((tab: SettingsTab = 'global'): void => {
		setActiveSettingsTab(tab);
	}, []);

	const closeSettings = useCallback((): void => {
		setActiveSettingsTab(null);
	}, []);

	const openLevelingGuideSettings = useCallback((): void => {
		openSettings('levelingGuide');
	}, [openSettings]);

	const openPobImportSettings = useCallback((): void => {
		openSettings('pobImport');
	}, [openSettings]);

	const openTimerSettings = useCallback((): void => {
		openSettings('timers');
	}, [openSettings]);

	if (viewMode === 'overlay') {
		const overlaySize = getOverlayLogicalSize(currentPage);
		return (
			<OverlayPanel logicalWidthPx={overlaySize.widthPx} logicalHeightPx={overlaySize.heightPx}>
				<LevelingGuideOverlay
					page={currentPage}
					loading={loading}
					error={error}
					onNavigate={handleNavigate}
					timerSettings={timerSettings}
					timerState={timerState}
					onTimerAction={handleTimerAction}
				/>
			</OverlayPanel>
		);
	}

	if (viewMode === 'hintTooltip') {
		return <HintTooltipView />;
	}

	const settingsContent =
		activeSettingsTab !== null ? (
			<SettingsPage
				activeTab={activeSettingsTab}
				onTabChange={setActiveSettingsTab}
				onBack={closeSettings}
				onResetAppData={wipeSettings}
				levelingGuideContent={
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
					/>
				}
				pobImportContent={
					<PobImportSettingsPanel
						pobSettings={pobSettings}
						loading={pobSettingsLoading}
						onAddSlot={addPobSlot}
						onRemoveSlot={removePobSlot}
						onSetCurrentSlot={setCurrentPobSlot}
					/>
				}
				timerContent={
					<TimerSettingsPanel
						actTimerEnabled={timerSettings.actTimerEnabled}
						campaignTimerEnabled={timerSettings.campaignTimerEnabled}
						onActTimerEnabledChange={updateTimerActEnabled}
						onCampaignTimerEnabledChange={updateTimerCampaignEnabled}
						settingsLoading={settingsLoading}
					/>
				}
			/>
		) : undefined;

	return (
		<MainView versionLabel={appVersion} settingsContent={settingsContent} onOpenSettings={() => openSettings('global')}>
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
			<PobImportDashboardSnippet pobSettings={pobSettings} onOpenSettings={openPobImportSettings} />
			<TimerDashboardSnippet timerSettings={timerSettings} timerState={timerState} onOpenSettings={openTimerSettings} />
			<ModuleSnippet
				title="Map Tracking"
				disabled
				hint="Track completed maps and their content."
				action={{ type: 'comingSoon' }}
				onSettingsClick={() => {}}
				settingsDisabled
			/>
			<ModuleSnippet
				title="Price Check"
				disabled
				hint="Quickly check item prices in-game."
				action={{ type: 'comingSoon' }}
				onSettingsClick={() => {}}
				settingsDisabled
			/>
			<ModuleSnippet
				title="Cheatsheets"
				disabled
				hint="Reference sheets for recipes, mechanics, and more."
				action={{ type: 'comingSoon' }}
				onSettingsClick={() => {}}
				settingsDisabled
			/>
		</MainView>
	);
}

export default App;
