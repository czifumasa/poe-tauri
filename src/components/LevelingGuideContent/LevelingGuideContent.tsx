import { Fragment, JSX, useRef, type PointerEvent as ReactPointerEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LevelingGuideLineDto, LevelingGuidePageDto, LevelingGuideSpanDto } from '../../types/Guide.ts';
import { requestOverlayFocus } from '../OverlayPanel/OverlayPanel';

import './LevelingGuideContent.css';

type OverlayLevelingGuideContentProps = {
	variant: 'overlay';
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onNavigate: (direction: 'previous' | 'next') => Promise<void>;
};

type DashboardLevelingGuideContentProps = {
	variant: 'dashboard';
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onLoadGuide: () => Promise<void>;
	onResetProgress: () => Promise<void>;
};

type LevelingGuideContentProps = OverlayLevelingGuideContentProps | DashboardLevelingGuideContentProps;

function renderSpan(span: LevelingGuideSpanDto, key: string): JSX.Element {
	if (span.type === 'image') {
		return <img key={key} className="guideInlineImage" src={span.dataUri} alt={span.key} />;
	}
	return <Fragment key={key}>{span.text}</Fragment>;
}

function renderLine(line: LevelingGuideLineDto, lineIndex: number): JSX.Element {
	const lineClassName = line.isHint ? 'guideStep guideStepHint' : 'guideStep';
	return (
		<div key={lineIndex} className={lineClassName}>
			{line.spans.map((span, spanIndex) => renderSpan(span, `${lineIndex}-${spanIndex}`))}
		</div>
	);
}

function getActLabel(page: LevelingGuidePageDto): string {
	return `ACT ${page.position.actIndex + 1}`;
}

function getPageCounterLabel(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

function getDashboardHeaderLabel(page: LevelingGuidePageDto): string {
	const actLabel = `Act ${page.position.actIndex + 1}`;
	return `${actLabel} - Page ${page.position.pageIndex + 1}/${page.pageCountInAct}`;
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

function OverlayGuideContent(props: OverlayLevelingGuideContentProps): JSX.Element {
	const { page, loading, onNavigate } = props;
	const dragStateRef = useRef<OverlayDragState | null>(null);
	const animationFrameIdRef = useRef<number | null>(null);
	const pendingPositionRef = useRef<OverlayPositionDto | null>(null);
	const lastAppliedPositionRef = useRef<OverlayPositionDto | null>(null);
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
		void requestOverlayFocus();
	};

	const handleNext = (): void => {
		void onNavigate('next');
		void requestOverlayFocus();
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
			lastAppliedPositionRef.current = position;
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
		void requestOverlayFocus();
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
		const position = computeDraggedPosition({ startPosition: state.startPosition, deltaX, deltaY });
		pendingPositionRef.current = position;
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
		lastAppliedPositionRef.current = finalPosition;

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

function DashboardGuideContent(props: DashboardLevelingGuideContentProps): JSX.Element {
	const { page, loading } = props;
	if (page === null) {
		return (
			<div className="guideNotLoaded">
				<div className="overlayMessage">Guide is not initialized.</div>
				{props.error && <div className="overlayError">{props.error}</div>}
				{loading && <div className="overlayLoading">Loading guide...</div>}
				<div className="guideDashboardControls">
					<button type="button" className="loadGuideButton" onClick={() => void props.onLoadGuide()} disabled={loading}>
						Load Guide
					</button>
					<button type="button" onClick={() => void props.onResetProgress()} disabled>
						Reset
					</button>
				</div>
			</div>
		);
	}

	return (
		<div className="guideContent guideContentCompact">
			<div className="guideHeader">{getDashboardHeaderLabel(page)}</div>
			<div className="guideNavigation">
				<button type="button" onClick={() => void props.onLoadGuide()} disabled={loading}>
					Load
				</button>
				<button type="button" onClick={() => void props.onResetProgress()} disabled={loading}>
					Reset
				</button>
			</div>
		</div>
	);
}

export function LevelingGuideContent({ page, loading, error, ...rest }: LevelingGuideContentProps): JSX.Element {
	if (rest.variant === 'overlay') {
		return (
			<OverlayGuideContent variant="overlay" page={page} loading={loading} error={error} onNavigate={rest.onNavigate} />
		);
	}

	return (
		<DashboardGuideContent
			variant="dashboard"
			page={page}
			loading={loading}
			error={error}
			onLoadGuide={rest.onLoadGuide}
			onResetProgress={rest.onResetProgress}
		/>
	);
}
