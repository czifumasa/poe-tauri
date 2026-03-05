export type OverlayPosition = {
	x: number;
	y: number;
};

export type BanditsChoice = 'KillAll' | 'HelpAlira' | 'HelpOak' | 'HelpKraityn';

export type LevelingGuideSettings = {
	leagueStart: boolean;
	overlayPosition: OverlayPosition | null;
	optionalQuests: boolean;
	levelRecommendations: boolean;
	banditsChoice: BanditsChoice;
	clientLogPath: string | null;
	gemsEnabled: boolean;
};

export type PobSlot = {
	pobCode: string;
	class: string;
	gemCount: number;
};

export type PobSettings = {
	slots: PobSlot[];
	currentSlotIndex: number | null;
};
