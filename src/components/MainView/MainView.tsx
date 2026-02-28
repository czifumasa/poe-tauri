import { JSX, type ReactNode, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

import { TitleBar } from '../TitleBar/TitleBar.tsx';
import { LeagueCard } from '../DashboardCards/LeagueCard.tsx';
import { CharacterCard } from '../DashboardCards/CharacterCard.tsx';
import { OverlaysCard } from '../DashboardCards/OverlaysCard.tsx';

import './MainView.css';

interface MainViewProps {
	children: ReactNode;
}

export function MainView({ children }: MainViewProps): JSX.Element {
	const [overlaysVisible, setOverlaysVisible] = useState<boolean>(false);

	const showAllOverlays = useCallback(async (): Promise<void> => {
		await invoke('show_overlay');
		setOverlaysVisible(true);
	}, []);

	const hideAllOverlays = useCallback(async (): Promise<void> => {
		await invoke('hide_overlay');
		setOverlaysVisible(false);
	}, []);

	const handleLeagueConfigure = useCallback((): void => {
		// placeholder for league configuration
	}, []);

	const handleCharacterConfigure = useCallback((): void => {
		// placeholder for character configuration
	}, []);

	return (
		<main className="mainViewContainer">
			<TitleBar version="V0.1.0" />

			<div className="mainViewTopRow">
				<LeagueCard leagueName="Settlers" leagueDetail="Hardcore · SSF" onConfigure={handleLeagueConfigure} />
				<CharacterCard characterName="Exile" characterDetail="Witch · Level 1" onConfigure={handleCharacterConfigure} />
				<OverlaysCard allVisible={overlaysVisible} onShowAll={() => void showAllOverlays()} onHideAll={() => void hideAllOverlays()} />
			</div>

			<div className="mainViewModulesDivider">
				<span className="mainViewModulesDividerLabel">MODULES</span>
			</div>

			<div className="mainViewModulesGrid">{children}</div>
		</main>
	);
}
