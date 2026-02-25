import { Fragment, JSX, type PointerEvent as ReactPointerEvent, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LevelingGuideLineDto, LevelingGuidePageDto, LevelingGuideSpanDto } from '../../../types/Guide.ts';

import '../LevelingGuideCommon.css';
import './LevelingGuideOverlay.css';

type LevelingGuideOverlayProps = {
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onNavigate: (direction: 'previous' | 'next') => Promise<void>;
};

function renderSpan(span: LevelingGuideSpanDto, key: string): JSX.Element {
	if (span.type === 'image') {
		return <img key={key} className="guideInlineImage" src={span.dataUri} alt={span.key} />;
	}
	if (span.color) {
		return (
			<span key={key} style={{ color: span.color }}>
				{span.text}
			</span>
		);
	}
	return <Fragment key={key}>{span.text}</Fragment>;
}

function renderLine(line: LevelingGuideLineDto, lineIndex: number): JSX.Element {
	const lineClassName = line.isHint ? 'guideStep guideStepHint' : 'guideStep';
	return (
		<div key={lineIndex} className={lineClassName}>
			<span>
				{line.spans.map((span, spanIndex) => renderSpan(span, `${lineIndex}-${spanIndex}`))}
			</span>
		</div>
	);
}

function getActLabel(page: LevelingGuidePageDto): string {
	return `ACT ${page.position.actIndex + 1}`;
}

function getPageCounterLabel(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

type OverlayPositionDto =
	| { type: 'absolute'; x: number; y: number }
	| { type: 'layer_shell_margins'; left: number; bottom: number };

type OverlayDragState =
	| {
			status: 'pending';
			pointerId: number;
			startClientX: number;
			startClientY: number;
	  }
	| {
			status: 'dragging';
			pointerId: number;
			startClientX: number;
			startClientY: number;
			startPosition: OverlayPositionDto;
	  };

function computeDraggedPosition(params: {
	startPosition: OverlayPositionDto;
	deltaX: number;
	deltaY: number;
}): OverlayPositionDto {
	if (params.startPosition.type === 'absolute') {
		return {
			type: 'absolute',
			x: Math.round(params.startPosition.x + params.deltaX),
			y: Math.round(params.startPosition.y + params.deltaY),
		};
	}

	return {
		type: 'layer_shell_margins',
		left: Math.max(0, Math.round(params.startPosition.left + params.deltaX)),
		bottom: Math.max(0, Math.round(params.startPosition.bottom - params.deltaY)),
	};
}

export function LevelingGuideOverlay(props: LevelingGuideOverlayProps): JSX.Element {
	const { page, loading, onNavigate } = props;
	const dragStateRef = useRef<OverlayDragState | null>(null);
	const animationFrameIdRef = useRef<number | null>(null);
	const pendingPositionRef = useRef<OverlayPositionDto | null>(null);
	const isApplyingRef = useRef<boolean>(false);

	if (page === null) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{props.error && <div className="overlayError">{props.error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
			</div>
		);
	}

	const handlePrevious = (): void => {
		void onNavigate('previous');
	};

	const handleNext = (): void => {
		void onNavigate('next');
	};

	const scheduleApply = (): void => {
		if (animationFrameIdRef.current !== null) {
			return;
		}

		animationFrameIdRef.current = window.requestAnimationFrame(() => {
			animationFrameIdRef.current = null;
			const position = pendingPositionRef.current;
			if (position === null) {
				return;
			}

			if (isApplyingRef.current) {
				scheduleApply();
				return;
			}

			isApplyingRef.current = true;
			void invoke('overlay_apply_position', { position })
				.catch((err: unknown) => {
					console.error('Failed to apply overlay position:', err);
				})
				.finally(() => {
					isApplyingRef.current = false;
				});
		});
	};

	const handleHeaderPointerDown = (event: ReactPointerEvent<HTMLDivElement>): void => {
		event.preventDefault();
		event.currentTarget.setPointerCapture(event.pointerId);
		dragStateRef.current = {
			status: 'pending',
			pointerId: event.pointerId,
			startClientX: event.clientX,
			startClientY: event.clientY,
		};

		void invoke<OverlayPositionDto>('overlay_get_position')
			.then((startPosition) => {
				const state = dragStateRef.current;
				if (state === null) {
					return;
				}
				if (state.status !== 'pending') {
					return;
				}
				dragStateRef.current = {
					status: 'dragging',
					pointerId: state.pointerId,
					startClientX: state.startClientX,
					startClientY: state.startClientY,
					startPosition,
				};
			})
			.catch((err: unknown) => {
				console.error('Failed to get overlay position:', err);
				dragStateRef.current = null;
			});
	};

	const handleHeaderPointerMove = (event: ReactPointerEvent<HTMLDivElement>): void => {
		const state = dragStateRef.current;
		if (state === null) {
			return;
		}
		if (state.pointerId !== event.pointerId) {
			return;
		}
		if (state.status !== 'dragging') {
			return;
		}

		const deltaX = event.clientX - state.startClientX;
		const deltaY = event.clientY - state.startClientY;

		pendingPositionRef.current = computeDraggedPosition({ startPosition: state.startPosition, deltaX, deltaY });
		scheduleApply();
	};

	const endDragging = (event: ReactPointerEvent<HTMLDivElement>): void => {
		const state = dragStateRef.current;
		dragStateRef.current = null;
		pendingPositionRef.current = null;

		if (animationFrameIdRef.current !== null) {
			window.cancelAnimationFrame(animationFrameIdRef.current);
			animationFrameIdRef.current = null;
		}

		if (state === null) {
			return;
		}
		if (state.pointerId !== event.pointerId) {
			return;
		}

		if (state.status !== 'dragging') {
			return;
		}

		const deltaX = event.clientX - state.startClientX;
		const deltaY = event.clientY - state.startClientY;
		const finalPosition = computeDraggedPosition({ startPosition: state.startPosition, deltaX, deltaY });

		void invoke('overlay_set_position', { position: finalPosition }).catch((err: unknown) => {
			console.error('Failed to persist overlay position:', err);
		});
	};

	return (
		<div className="guideContent guideContentOverlay">
			<div
				className="guideHeader guideHeaderOverlay"
				onPointerDown={handleHeaderPointerDown}
				onPointerMove={handleHeaderPointerMove}
				onPointerUp={endDragging}
				onPointerCancel={endDragging}
				style={{ touchAction: 'none', cursor: 'move' }}>
				<div className="guideHeaderLeft">{getActLabel(page)}</div>
				<div className="guideHeaderRight">{getPageCounterLabel(page)}</div>
			</div>
			<div className="guideSteps">{page.lines.map((line, lineIndex) => renderLine(line, lineIndex))}</div>
			<div className="guideNavigation">
				<button
					type="button"
					className="guideNavButton"
					onClick={handlePrevious}
					disabled={!page.hasPrevious || loading}>
					{'← PREV'}
				</button>
				<button type="button" className="guideNavButton" onClick={handleNext} disabled={!page.hasNext || loading}>
					{'NEXT →'}
				</button>
			</div>
		</div>
	);
}
