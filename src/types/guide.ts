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

export interface GuidePosition {
	actIndex: number;
	pageIndex: number;
}

export interface LevelingGuidePageDto {
	guidePath: string;
	position: GuidePosition;
	actCount: number;
	pageCountInAct: number;
	lines: string[];
	hasPrevious: boolean;
	hasNext: boolean;
}

export interface GuideState {
	guide: Guide | null;
	currentAct: number;
	currentPage: number;
	loading: boolean;
	error: string | null;
}
