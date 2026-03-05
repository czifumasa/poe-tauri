import { JSX } from 'react';
import type { TimerSettings, TimerState } from '../../../types/Timer.ts';
import { formatElapsedMs } from '../../../utils/formatTime.ts';
import { ModuleSnippet } from '../../ModuleSnippet/ModuleSnippet.tsx';

import './TimerDashboardSnippet.css';

const ACT_COUNT = 10;

type TimerDashboardSnippetProps = {
	timerSettings: TimerSettings;
	timerState: TimerState;
	onOpenSettings: () => void;
	onViewDetails: () => void;
	onSaveRun: () => void;
	onResetRun: () => void;
};

function SaveIcon(): JSX.Element {
	return (
		<svg
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
			<polyline points="17 21 17 13 7 13 7 21" />
			<polyline points="7 3 7 8 15 8" />
		</svg>
	);
}

function ResetIcon(): JSX.Element {
	return (
		<svg
			width="16"
			height="16"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<polyline points="23 4 23 10 17 10" />
			<path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
		</svg>
	);
}

const HALF = Math.ceil(ACT_COUNT / 2);

function TimerActCell(props: { actIndex: number; timerState: TimerState }): JSX.Element {
	const { actIndex, timerState } = props;
	const elapsed = timerState.actElapsedMs[actIndex] ?? 0;
	const isActive = actIndex === timerState.currentActIndex && timerState.status !== 'idle';
	const isCompleted = !isActive && actIndex < timerState.currentActIndex && timerState.status !== 'idle';
	const displayMs = isActive ? timerState.currentActElapsedMs : elapsed;
	const cellClass = isActive
		? 'timerSnippetCell timerSnippetCell--active'
		: isCompleted
			? 'timerSnippetCell timerSnippetCell--completed'
			: 'timerSnippetCell';

	return (
		<div className={cellClass}>
			<span className="timerSnippetActLabel">Act {actIndex + 1}</span>
			<span className="timerSnippetTimeLabel">
				{isActive || isCompleted ? formatElapsedMs(displayMs) : '--:--:--'}
			</span>
		</div>
	);
}

function TimerTimetable(props: { timerState: TimerState }): JSX.Element {
	const { timerState } = props;
	const leftActs = Array.from({ length: HALF }, (_, i) => i);
	const rightActs = Array.from({ length: ACT_COUNT - HALF }, (_, i) => i + HALF);

	return (
		<div className="timerSnippetTimetable">
			<div className="timerSnippetColumn">
				{leftActs.map((actIndex) => (
					<TimerActCell key={actIndex} actIndex={actIndex} timerState={timerState} />
				))}
			</div>
			<div className="timerSnippetDivider" />
			<div className="timerSnippetColumn">
				{rightActs.map((actIndex) => (
					<TimerActCell key={actIndex} actIndex={actIndex} timerState={timerState} />
				))}
			</div>
			<div className="timerSnippetTotal">
				<span className="timerSnippetActLabel">Total</span>
				<span className="timerSnippetTimeLabel">{formatElapsedMs(timerState.campaignElapsedMs)}</span>
			</div>
		</div>
	);
}

export function TimerDashboardSnippet(props: TimerDashboardSnippetProps): JSX.Element {
	const { timerSettings, timerState } = props;
	const isActive = timerSettings.enabled;
	const hasRunData = timerState.status !== 'idle';

	return (
		<ModuleSnippet
			title="Run Timer"
			active={isActive}
			hint={isActive ? undefined : 'Timer is disabled. Enable it in settings.'}
			onSettingsClick={props.onOpenSettings}
			action={
				isActive
					? [
							{ type: 'primary', label: 'DETAILS', onClick: props.onViewDetails },
							{ type: 'icon', icon: <SaveIcon />, title: 'Save Run', onClick: props.onSaveRun, disabled: !hasRunData },
							{
								type: 'icon',
								icon: <ResetIcon />,
								title: 'Reset Run',
								onClick: props.onResetRun,
								disabled: !hasRunData,
							},
						]
					: undefined
			}>
			<div className="timerSnippetBody">{isActive && <TimerTimetable timerState={timerState} />}</div>
		</ModuleSnippet>
	);
}
