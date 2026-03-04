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

export type SavedRun = {
	readonly id: string;
	readonly name: string;
	readonly league: string;
	readonly character: string;
	readonly characterClass: string;
	readonly actElapsedMs: readonly number[];
	readonly campaignElapsedMs: number;
	readonly savedAt: number;
};
