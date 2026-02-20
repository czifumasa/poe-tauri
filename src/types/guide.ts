export interface GuideCondition {
	type: string;
	value: string | string[];
}

export interface ConditionalPage {
	condition: [string, string | string[]];
	lines: string[];
}

export type GuidePage = string[] | ConditionalPage;

export type GuideAct = GuidePage[];

export type Guide = GuideAct[];

export interface GuideState {
	guide: Guide | null;
	currentAct: number;
	currentPage: number;
	loading: boolean;
	error: string | null;
}
