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

function TimerTimetable(props: { timerState: TimerState }): JSX.Element {
	const { timerState } = props;
	const acts = Array.from({ length: ACT_COUNT }, (_, i) => i);

	return (
		<div className="timerSnippetTimetable">
			{acts.map((actIndex) => {
				const elapsed = timerState.actElapsedMs[actIndex] ?? 0;
				const isActive = actIndex === timerState.currentActIndex && timerState.status !== 'idle';
				const displayMs = isActive ? timerState.currentActElapsedMs : elapsed;
				const cellClass = isActive ? 'timerSnippetCell timerSnippetCell--active' : 'timerSnippetCell';

				return (
					<div key={actIndex} className={cellClass}>
						<span className="timerSnippetActLabel">Act {actIndex + 1}</span>
						<span className="timerSnippetTimeLabel">{formatElapsedMs(displayMs)}</span>
					</div>
				);
			})}
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
			onSettingsClick={props.onOpenSettings}>
			<div className="timerSnippetBody">{isActive && <TimerTimetable timerState={timerState} />}</div>
		</ModuleSnippet>
	);
}
