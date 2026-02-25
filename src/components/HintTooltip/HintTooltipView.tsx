import { JSX, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { HINT_TOOLTIP_VIEW_QUERY_VALUE } from '../../constants/WindowIdentifiers.ts';

type HintTooltipContentPayload = {
	key: string;
	dataUri: string;
};

export function HintTooltipView(): JSX.Element {
	const [content, setContent] = useState<HintTooltipContentPayload | null>(null);

	useEffect((): (() => void) => {
		document.documentElement.dataset.view = HINT_TOOLTIP_VIEW_QUERY_VALUE;
		return (): void => {
			delete document.documentElement.dataset.view;
		};
	}, []);

	useEffect((): (() => void) => {
		let isDisposed = false;
		let unlisten: (() => void) | null = null;

		void (async (): Promise<void> => {
			try {
				unlisten = await listen<HintTooltipContentPayload>('hint_tooltip_content', (event) => {
					if (isDisposed) {
						return;
					}
					setContent(event.payload);
				});

				const last = await invoke<HintTooltipContentPayload | null>('hint_tooltip_get_last_content');
				if (!isDisposed && last !== null) {
					setContent(last);
				}
			} catch (err) {
				console.error('Failed to listen for hint tooltip updates:', err);
			}
		})();

		return (): void => {
			isDisposed = true;
			if (unlisten !== null) {
				unlisten();
			}
		};
	}, []);

	return (
		<main
			style={{
				width: '100vw',
				height: '100vh',
				display: 'flex',
				alignItems: 'center',
				justifyContent: 'center',
				background: 'transparent',
				overflow: 'hidden',
			}}>
			{content && (
				<img
					src={content.dataUri}
					alt={content.key}
					style={{
						maxWidth: '100%',
						maxHeight: '100%',
						display: 'block',
						objectFit: 'contain',
					}}
				/>
			)}
		</main>
	);
}
