import { JSX, type ReactNode, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { OVERLAY_VIEW_QUERY_VALUE } from '../../constants/WindowIdentifiers.ts';

import './OverlayPanel.css';

interface OverlayPanelProps {
	children: ReactNode;
	logicalWidthPx: number;
	logicalHeightPx: number;
}

export function OverlayPanel({ children, logicalWidthPx, logicalHeightPx }: OverlayPanelProps): JSX.Element {
	const overlayPanelRef = useRef<HTMLDivElement | null>(null);

	useEffect((): (() => void) => {
		document.documentElement.dataset.view = OVERLAY_VIEW_QUERY_VALUE;
		const devicePixelRatio = window.devicePixelRatio || 1;
		const width = Math.max(1, Math.ceil(logicalWidthPx * devicePixelRatio));
		const height = Math.max(1, Math.ceil(logicalHeightPx * devicePixelRatio));
		void invoke('set_overlay_panel_size', { width, height });

		return (): void => {
			delete document.documentElement.dataset.view;
		};
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
