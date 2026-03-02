import { JSX, useEffect, useLayoutEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { HINT_TOOLTIP_VIEW_QUERY_VALUE } from '../../constants/WindowIdentifiers.ts';

type HintTooltipContentPayload = {
	key: string;
	dataUri: string;
};

export function HintTooltipView(): JSX.Element {
	const [content, setContent] = useState<HintTooltipContentPayload | null>(null);

	const resetThenSetContent = (next: HintTooltipContentPayload): void => {
		setContent(null);
		requestAnimationFrame(() => {
			setContent(next);
		});
	};

	useLayoutEffect((): (() => void) => {
		document.documentElement.dataset.view = HINT_TOOLTIP_VIEW_QUERY_VALUE;
		return (): void => {
			delete document.documentElement.dataset.view;
		};
	}, []);

	useEffect((): (() => void) => {
		let isDisposed = false;
		let unlisten: (() => void) | null = null;
		let unlistenClear: (() => void) | null = null;

		void (async (): Promise<void> => {
			try {
				unlisten = await listen<HintTooltipContentPayload>('hint_tooltip_content', (event) => {
					if (isDisposed) {
						return;
					}
					resetThenSetContent(event.payload);
				});

				unlistenClear = await listen('hint_tooltip_clear', () => {
					if (isDisposed) {
						return;
					}
					setContent(null);
				});

				const last = await invoke<HintTooltipContentPayload | null>('hint_tooltip_get_last_content');
				if (!isDisposed && last !== null) {
					resetThenSetContent(last);
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
			if (unlistenClear !== null) {
				unlistenClear();
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
					key={content.key}
					src={content.dataUri}
					alt={content.key}
					style={{
						width: '100%',
						height: '100%',
						display: 'block',
						objectFit: 'contain',
					}}
				/>
			)}
		</main>
	);
}
