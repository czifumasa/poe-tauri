export type TimerStatus = 'idle' | 'running' | 'paused';

export type TimerSettings = {
	actTimerEnabled: boolean;
	campaignTimerEnabled: boolean;
};

export type TimerState = {
	status: TimerStatus;
	currentActIndex: number;
	actElapsedMs: number[];
	currentActElapsedMs: number;
	campaignElapsedMs: number;
};
