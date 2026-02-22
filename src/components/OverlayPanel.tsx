import { JSX, ReactNode, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { OVERLAY_VIEW_QUERY_VALUE } from '../windowIdentifiers';

interface OverlayPanelProps {
	children: ReactNode;
}

export function OverlayPanel({ children }: OverlayPanelProps): JSX.Element {
	const overlayPanelRef = useRef<HTMLDivElement | null>(null);
	const lastReportedSizeRef = useRef<{ width: number; height: number } | null>(null);
	const reportTimeoutIdRef = useRef<number | null>(null);

	async function releaseOverlayFocus(): Promise<void> {
		await invoke('set_overlay_interactive', { interactive: false });
	}

	useEffect((): (() => void) => {
		document.documentElement.dataset.view = OVERLAY_VIEW_QUERY_VALUE;
		let isDisposed = false;

		const reportPanelSizeAfterLayout = (): void => {
			if (reportTimeoutIdRef.current !== null) {
				window.clearTimeout(reportTimeoutIdRef.current);
				reportTimeoutIdRef.current = null;
			}

			reportTimeoutIdRef.current = window.setTimeout(() => {
				window.requestAnimationFrame(() => {
					if (isDisposed) {
						return;
					}
					const element = overlayPanelRef.current;
					if (element === null) {
						return;
					}
					const rect = element.getBoundingClientRect();
					const devicePixelRatio = window.devicePixelRatio || 1;
					const width = Math.max(1, Math.round(rect.width * devicePixelRatio));
					const height = Math.max(1, Math.round(rect.height * devicePixelRatio));
					const last = lastReportedSizeRef.current;
					const hysteresisPixels = 2;
					if (
						last !== null &&
						Math.abs(last.width - width) < hysteresisPixels &&
						Math.abs(last.height - height) < hysteresisPixels
					) {
						return;
					}
					lastReportedSizeRef.current = { width, height };
					void invoke('set_overlay_panel_size', { width, height });
				});
			}, 60);
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
			if (reportTimeoutIdRef.current !== null) {
				window.clearTimeout(reportTimeoutIdRef.current);
				reportTimeoutIdRef.current = null;
			}
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
