import { JSX } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function MainView(): JSX.Element {
	async function openOverlay(): Promise<void> {
		await invoke('show_overlay');
	}

	async function hideOverlay(): Promise<void> {
		await invoke('hide_overlay');
	}

	return (
		<main className="container">
			<h1>Poe Tauri</h1>

			<div className="row overlayControls">
				<button type="button" onClick={() => void openOverlay()}>
					Show Overlay
				</button>
				<button type="button" onClick={() => void hideOverlay()}>
					Hide Overlay
				</button>
			</div>
		</main>
	);
}
