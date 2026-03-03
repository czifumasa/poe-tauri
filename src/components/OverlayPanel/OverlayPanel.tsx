import { JSX, type ReactNode, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './OverlayPanel.css';

interface OverlayPanelProps {
	children: ReactNode;
	logicalWidthPx: number;
	logicalHeightPx: number;
}

export function OverlayPanel({ children, logicalWidthPx, logicalHeightPx }: OverlayPanelProps): JSX.Element {
	const overlayPanelRef = useRef<HTMLDivElement | null>(null);

	useEffect((): void => {
		const devicePixelRatio = window.devicePixelRatio || 1;
		const width = Math.max(1, Math.ceil(logicalWidthPx * devicePixelRatio));
		const height = Math.max(1, Math.ceil(logicalHeightPx * devicePixelRatio));
		void invoke('set_overlay_panel_size', { width, height });
	}, [logicalWidthPx, logicalHeightPx]);

	return (
		<main className="overlayContainer">
			<div
				className="overlayPanel overlayPanelBorder"
				ref={overlayPanelRef}
				style={{ width: `${logicalWidthPx}px`, height: `${logicalHeightPx}px` }}>
				{children}
			</div>
		</main>
	);
}
