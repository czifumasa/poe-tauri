export type OverlayPosition = {
	x: number;
	y: number;
};

export type BanditsChoice = 'KillAll' | 'HelpAlira' | 'HelpOak' | 'HelpKraityn';

export type LevelingGuideSettings = {
	schemaVersion: number;
	leagueStart: boolean;
	overlayPosition: OverlayPosition | null;
	optionalQuests: boolean;
	levelRecommendations: boolean;
	banditsChoice: BanditsChoice;
	clientLogPath: string | null;
	gemsEnabled: boolean;
	overlayShown: boolean;
};

export type PobSlot = {
	pobCode: string;
	class: string;
	ascendClass: string | null;
	gemCount: number;
};

function isNonEmptyAscendClass(value: string | null): value is string {
	return value !== null && value !== '' && value.toLowerCase() !== 'none';
}

function titleCase(value: string): string {
	return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
}

export function pobSlotDisplayClass(slot: PobSlot): string {
	const raw = isNonEmptyAscendClass(slot.ascendClass) ? slot.ascendClass : slot.class;
	return titleCase(raw);
}

export type PobSettings = {
	schemaVersion: number;
	slots: PobSlot[];
	currentSlotIndex: number | null;
};
