import { JSX, ReactNode, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface OverlayPanelProps {
	children: ReactNode;
}

export function OverlayPanel({ children }: OverlayPanelProps): JSX.Element {
	const overlayPanelRef = useRef<HTMLDivElement | null>(null);
	const lastReportedSizeRef = useRef<{ width: number; height: number } | null>(null);

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
			if (!document.hasFocus()) {
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
				{children}
			</div>
		</main>
	);
}

export async function requestOverlayFocus(): Promise<void> {
	await invoke('set_overlay_interactive', { interactive: true });
}
