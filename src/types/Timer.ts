export type TimerStatus = 'idle' | 'running' | 'paused';

export type TimerSettings = {
	schemaVersion: number;
	enabled: boolean;
	displayActTimer: boolean;
	displayCampaignTimer: boolean;
};

export type TimerState = {
	schemaVersion: number;
	status: TimerStatus;
	runId: string | null;
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

export type SavedRunStatus = 'completed' | 'in_progress';

export type SavedRun = {
	readonly schemaVersion: number;
	readonly id: string;
	readonly league: string;
	readonly hardcore: boolean;
	readonly ssf: boolean;
	readonly privateLeague: boolean;
	readonly character: string;
	readonly characterClass: string;
	readonly runDetails: string;
	readonly status: SavedRunStatus;
	readonly actRuns: readonly ActRun[];
	readonly campaignElapsedMs: number;
	readonly savedAt: number;
};
