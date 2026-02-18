import { JSX, useEffect, useMemo, useRef, useState } from 'react';
import reactLogo from './assets/react.svg';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

type ViewModeWithOverlayPanel = 'main' | 'overlay' | 'overlay_panel';

function getViewMode(): ViewModeWithOverlayPanel {
	const params = new URLSearchParams(window.location.search);
	const view = params.get('view');
	if (view === 'overlay') {
		return 'overlay';
	}
	if (view === 'overlay-panel') {
		return 'overlay_panel';
	}
	return 'main';
}

function MainView(): JSX.Element {
	const [greetMsg, setGreetMsg] = useState<string>('');
	const [name, setName] = useState<string>('');

	async function greet(): Promise<void> {
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		const message = await invoke<string>('greet', { name });
		setGreetMsg(message);
	}

	async function openOverlay(): Promise<void> {
		await invoke('show_overlay');
	}

	async function hideOverlay(): Promise<void> {
		await invoke('hide_overlay');
	}

	return (
		<main className="container">
			<h1>Welcome to Tauri + React</h1>

			<div className="row">
				<a href="https://vite.dev" target="_blank">
					<img src="/vite.svg" className="logo vite" alt="Vite logo" />
				</a>
				<a href="https://tauri.app" target="_blank">
					<img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
				</a>
				<a href="https://react.dev" target="_blank">
					<img src={reactLogo} className="logo react" alt="React logo" />
				</a>
			</div>
			<p>Click on the Tauri, Vite, and React logos to learn more.</p>

			<form
				className="row"
				onSubmit={(e) => {
					e.preventDefault();
					void greet();
				}}>
				<input
					id="greet-input"
					onChange={(e) => setName(e.currentTarget.value)}
					placeholder="Enter a name..."
					value={name}
				/>
				<button type="submit">Greet</button>
			</form>

			<div className="row overlayControls">
				<button type="button" onClick={() => void openOverlay()}>
					Show overlay
				</button>
				<button type="button" onClick={() => void hideOverlay()}>
					Hide overlay
				</button>
			</div>

			<p>{greetMsg}</p>
		</main>
	);
}

function OverlayFullscreenView(): JSX.Element {
	useEffect((): (() => void) => {
		document.documentElement.dataset.view = 'overlay';
		return (): void => {
			delete document.documentElement.dataset.view;
		};
	}, []);

	return <main className="overlayContainer" />;
}

function OverlayPanelView(): JSX.Element {
	const overlayPanelRef = useRef<HTMLDivElement | null>(null);
	const lastReportedSizeRef = useRef<{ width: number; height: number } | null>(null);
	const [isFocused, setIsFocused] = useState<boolean>(() => document.hasFocus());
	const [clickCount, setClickCount] = useState<number>(0);
	const [lastClickMessage, setLastClickMessage] = useState<string>('No click yet.');

	async function requestOverlayFocus(): Promise<void> {
		await invoke('set_overlay_interactive', { interactive: true });
	}

	async function releaseOverlayFocus(): Promise<void> {
		await invoke('set_overlay_interactive', { interactive: false });
	}

	useEffect((): (() => void) => {
		document.documentElement.dataset.view = 'overlay';
		let isDisposed = false;

		const reportPanelSizeAfterLayout = (): void => {
			window.requestAnimationFrame(() => {
				if (isDisposed) {
					return;
				}
				const element = overlayPanelRef.current;
				if (element === null) {
					return;
				}
				const rect = element.getBoundingClientRect();
				const width = Math.ceil(rect.width);
				const height = Math.ceil(rect.height);
				const last = lastReportedSizeRef.current;
				if (last !== null && last.width === width && last.height === height) {
					return;
				}
				lastReportedSizeRef.current = { width, height };
				void invoke('set_overlay_panel_size', { width, height });
			});
		};

		reportPanelSizeAfterLayout();
		window.setTimeout(reportPanelSizeAfterLayout, 0);
		window.setTimeout(reportPanelSizeAfterLayout, 50);
		window.setTimeout(reportPanelSizeAfterLayout, 250);
		const onFocusChanged = (): void => {
			const focusedNow = document.hasFocus();
			setIsFocused(focusedNow);
			if (!focusedNow) {
				void releaseOverlayFocus();
			}
		};

		const resizeObserver = new ResizeObserver(() => {
			reportPanelSizeAfterLayout();
		});
		if (overlayPanelRef.current !== null) {
			resizeObserver.observe(overlayPanelRef.current);
		}
		window.addEventListener('focus', onFocusChanged);
		window.addEventListener('blur', onFocusChanged);
		const intervalId = window.setInterval(onFocusChanged, 250);
		return (): void => {
			isDisposed = true;
			resizeObserver.disconnect();
			window.clearInterval(intervalId);
			window.removeEventListener('focus', onFocusChanged);
			window.removeEventListener('blur', onFocusChanged);
			delete document.documentElement.dataset.view;
		};
	}, []);

	return (
		<main className="overlayContainer">
			<div className="overlayPanel overlayPanelBorder" ref={overlayPanelRef}>
				<div className="overlayTitle">poe-tauri overlay PoC</div>
				<div className="overlayRow">Panel state: {isFocused ? 'focused' : 'not focused'}</div>
				<div className="overlayRow">Clicks registered: {clickCount}</div>
				<div className="overlayRow">Last click: {lastClickMessage}</div>
				<div className="overlayRow">dpr: {window.devicePixelRatio}</div>
				<div className="overlayButtons">
					<button
						type="button"
						onClick={() => {
							setClickCount((prev) => prev + 1);
							setLastClickMessage(new Date().toISOString());
							void requestOverlayFocus();
						}}>
						Debug click
					</button>
				</div>
			</div>
		</main>
	);
}

function App(): JSX.Element {
	const viewMode = useMemo<ViewModeWithOverlayPanel>(() => getViewMode(), []);
	if (viewMode === 'overlay') {
		return <OverlayFullscreenView />;
	}
	if (viewMode === 'overlay_panel') {
		return <OverlayPanelView />;
	}
	return <MainView />;
}

export default App;
