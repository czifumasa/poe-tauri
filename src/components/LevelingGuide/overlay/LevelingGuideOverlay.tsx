import { Fragment, JSX, type PointerEvent as ReactPointerEvent, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { LevelingGuideLineDto, LevelingGuidePageDto, LevelingGuideSpanDto } from '../../../types/Guide.ts';
import type { TimerSettings, TimerState } from '../../../types/Timer.ts';
import { formatElapsedMs } from '../../../utils/formatTime.ts';

import '../LevelingGuideCommon.css';
import './LevelingGuideOverlay.css';

type TimerAction = 'start' | 'pause' | 'resume' | 'reset';

type LevelingGuideOverlayProps = {
	page: LevelingGuidePageDto | null;
	loading: boolean;
	error: string | null;
	onNavigate: (direction: 'previous' | 'next') => Promise<void>;
	timerSettings: TimerSettings;
	timerState: TimerState;
	onTimerAction: (action: TimerAction) => void;
};

function getActLabel(page: LevelingGuidePageDto): string {
	return `ACT ${page.position.actIndex + 1}`;
}

function getPageCounterLabel(page: LevelingGuidePageDto): string {
	return `${page.position.pageIndex + 1} / ${page.pageCountInAct}`;
}

function getCampaignCounterLabel(page: LevelingGuidePageDto): string {
	return `${page.campaignPageIndex + 1} / ${page.campaignPageCount}`;
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

	if (page === null) {
		return (
			<div className="guideContent guideContentOverlay">
				<div
					className="guideHeader guideHeaderOverlay"
					onPointerDown={handleHeaderPointerDown}
					onPointerMove={handleHeaderPointerMove}
					onPointerUp={endDragging}
					onPointerCancel={endDragging}
					style={{ touchAction: 'none', cursor: 'move' }}
				/>
				<div className="guideSteps">
					<div className="overlayMessage">Guide is not initialized.</div>
					{props.error && <div className="overlayError">{props.error}</div>}
					{loading && <div className="overlayLoading">Loading guide...</div>}
				</div>
			</div>
		);
	}

	const renderSpan = (span: LevelingGuideSpanDto, key: string): JSX.Element => {
		if (span.type === 'image') {
			return <img key={key} className="guideInlineImage" src={span.dataUri} alt={span.key} />;
		}

		const style = span.color ? { color: span.color } : undefined;
		if (span.hint) {
			const hint = span.hint;
			const handleEnter = (): void => {
				void invoke('hint_tooltip_show', {
					args: {
						key: hint.key,
						dataUri: hint.dataUri,
					},
				}).catch((err: unknown) => {
					console.error('Failed to show hint tooltip:', err);
				});
			};

			const handleLeave = (): void => {
				void invoke('hint_tooltip_hide').catch((err: unknown) => {
					console.error('Failed to hide hint tooltip:', err);
				});
			};

			return (
				<span
					key={key}
					className="guideHintText"
					style={style}
					onPointerEnter={handleEnter}
					onPointerLeave={handleLeave}>
					{span.text}
				</span>
			);
		}

		if (span.color) {
			return (
				<span key={key} style={style}>
					{span.text}
				</span>
			);
		}
		return <Fragment key={key}>{span.text}</Fragment>;
	};

	const renderLine = (line: LevelingGuideLineDto, lineIndex: number): JSX.Element => {
		const lineClassName = line.isHint ? 'guideStep guideStepHint' : 'guideStep';
		return (
			<div key={lineIndex} className={lineClassName}>
				<span>
					{!line.isHint && '• '}
					{line.spans.map((span, spanIndex) => renderSpan(span, `${lineIndex}-${spanIndex}`))}
				</span>
			</div>
		);
	};

	const handlePrevious = (): void => {
		void onNavigate('previous');
	};

	const handleNext = (): void => {
		void onNavigate('next');
	};

	const { timerSettings, timerState, onTimerAction } = props;
	const timerEnabled = timerSettings.enabled;

	const renderTimerControls = (): JSX.Element | null => {
		if (!timerEnabled) {
			return null;
		}

		if (timerState.status === 'idle') {
			return (
				<button
					type="button"
					className="guideTimerButton guideTimerButtonStart"
					title="Start timer"
					onClick={() => onTimerAction('start')}>
					{'▶'}
				</button>
			);
		}

		if (timerState.status === 'running') {
			return (
				<button
					type="button"
					className="guideTimerButton guideTimerButtonPause"
					title="Pause timer"
					onClick={() => onTimerAction('pause')}>
					{'❚❚'}
				</button>
			);
		}

		return (
			<button
				type="button"
				className="guideTimerButton guideTimerButtonResume"
				title="Resume timer"
				onClick={() => onTimerAction('resume')}>
				{'▶'}
			</button>
		);
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
				<div className="guideHeaderRow">
					<div className="guideHeaderLeft">{getActLabel(page)}</div>
					{timerEnabled && timerSettings.displayActTimer && (
						<div className="guideHeaderCenter">{formatElapsedMs(timerState.currentActElapsedMs)}</div>
					)}
					<div className="guideHeaderRight">{getPageCounterLabel(page)}</div>
				</div>
				<div className="guideHeaderRow guideHeaderRowCampaign">
					<div className="guideHeaderLeft">Campaign</div>
					{timerEnabled && timerSettings.displayCampaignTimer && (
						<div className="guideHeaderCenter">{formatElapsedMs(timerState.campaignElapsedMs)}</div>
					)}
					<div className="guideHeaderRight">{getCampaignCounterLabel(page)}</div>
				</div>
			</div>
			{timerEnabled && timerSettings.warnWhenPaused && timerState.status === 'paused' ? (
				<div className="guideSteps">
					<div className="overlayMessage">Timer is paused.</div>
				</div>
			) : (
				<div className="guideSteps">{page.lines.map((line, lineIndex) => renderLine(line, lineIndex))}</div>
			)}
			<div className="guideNavigation">
				<button
					type="button"
					className="guideNavButton"
					onClick={handlePrevious}
					disabled={!page.hasPrevious || loading}>
					{'← PREV'}
				</button>
				{renderTimerControls()}
				<button type="button" className="guideNavButton" onClick={handleNext} disabled={!page.hasNext || loading}>
					{'NEXT →'}
				</button>
			</div>
		</div>
	);
}
