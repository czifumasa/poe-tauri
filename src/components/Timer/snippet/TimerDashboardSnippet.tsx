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
};

const HALF = Math.ceil(ACT_COUNT / 2);

function TimerActCell(props: { actIndex: number; timerState: TimerState }): JSX.Element {
	const { actIndex, timerState } = props;
	const elapsed = timerState.actElapsedMs[actIndex] ?? 0;
	const isActive = actIndex === timerState.currentActIndex && timerState.status !== 'idle';
	const displayMs = isActive ? timerState.currentActElapsedMs : elapsed;
	const cellClass = isActive ? 'timerSnippetCell timerSnippetCell--active' : 'timerSnippetCell';

	return (
		<div className={cellClass}>
			<span className="timerSnippetActLabel">Act {actIndex + 1}</span>
			<span className="timerSnippetTimeLabel">{formatElapsedMs(displayMs)}</span>
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
	const isActive = timerSettings.actTimerEnabled || timerSettings.campaignTimerEnabled;

	return (
		<ModuleSnippet
			title="Timers"
			active={isActive}
			hint={isActive ? undefined : 'All timers are disabled. Enable act or campaign timer in settings.'}
			onSettingsClick={props.onOpenSettings}>
			<div className="timerSnippetBody">{isActive && <TimerTimetable timerState={timerState} />}</div>
		</ModuleSnippet>
	);
}
