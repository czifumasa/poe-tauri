export type TimerStatus = 'idle' | 'running' | 'paused';

export type TimerSettings = {
	enabled: boolean;
	displayActTimer: boolean;
	displayCampaignTimer: boolean;
};

export type TimerState = {
	status: TimerStatus;
	currentActIndex: number;
	actElapsedMs: number[];
	currentActElapsedMs: number;
	campaignElapsedMs: number;
};

export type ActRunStatus = 'completed' | 'in_progress' | 'pending';

export type ActRun = {
	readonly actName: string;
	readonly elapsedMs: number;
	readonly status: ActRunStatus;
};

export type SavedRun = {
	readonly id: string;
	readonly league: string;
	readonly hardcore: boolean;
	readonly ssf: boolean;
	readonly privateLeague: boolean;
	readonly character: string;
	readonly characterClass: string;
	readonly runDetails: string;
	readonly actRuns: readonly ActRun[];
	readonly campaignElapsedMs: number;
	readonly savedAt: number;
};
