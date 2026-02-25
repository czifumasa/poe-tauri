export interface GuidePosition {
	actIndex: number;
	pageIndex: number;
}

export type LevelingGuideSpanDto =
	| {
			type: 'text';
			text: string;
			color?: string;
			hint?: {
				key: string;
				dataUri: string;
			};
	  }
	| {
			type: 'image';
			key: string;
			dataUri: string;
	  };

export interface LevelingGuideLineDto {
	isHint: boolean;
	spans: LevelingGuideSpanDto[];
}

export interface LevelingGuidePageDto {
	guidePath: string;
	position: GuidePosition;
	actCount: number;
	pageCountInAct: number;
	lines: LevelingGuideLineDto[];
	hasPrevious: boolean;
	hasNext: boolean;
}
