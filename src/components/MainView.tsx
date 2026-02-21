import { JSX, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MainViewProps {
	children: ReactNode;
}

export function MainView({ children }: MainViewProps): JSX.Element {
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

			<div className="dashboardGrid">
				<section className="dashboardCard">
					<h2>Leveling Guide</h2>
					{children}
				</section>
			</div>
		</main>
	);
}
