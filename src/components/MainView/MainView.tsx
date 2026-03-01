import { JSX, type ReactNode, useCallback } from 'react';

import { TitleBar } from '../TitleBar/TitleBar.tsx';
import { LeagueCard } from '../DashboardCards/LeagueCard.tsx';
import { CharacterCard } from '../DashboardCards/CharacterCard.tsx';
import { OverlaysCard } from '../DashboardCards/OverlaysCard.tsx';

import './MainView.css';

interface MainViewProps {
	children: ReactNode;
	settingsContent?: ReactNode;
	overlaysVisible: boolean;
	onShowAllOverlays: () => Promise<void>;
	onHideAllOverlays: () => Promise<void>;
}

export function MainView({
	children,
	settingsContent,
	overlaysVisible,
	onShowAllOverlays,
	onHideAllOverlays,
}: MainViewProps): JSX.Element {

	const handleLeagueConfigure = useCallback((): void => {
		// placeholder for league configuration
	}, []);

	const handleCharacterConfigure = useCallback((): void => {
		// placeholder for character configuration
	}, []);

	return (
		<main className="mainViewContainer">
			<TitleBar version="v0.1.1" />

			{settingsContent !== undefined ? (
				<div className="mainViewSettingsArea">{settingsContent}</div>
			) : (
				<>
					<div className="mainViewTopRow">
						<LeagueCard leagueName="Settlers" leagueDetail="Hardcore · SSF" onConfigure={handleLeagueConfigure} />
						<CharacterCard
							characterName="Exile"
							characterDetail="Witch · Level 1"
							onConfigure={handleCharacterConfigure}
						/>
						<OverlaysCard
							allVisible={overlaysVisible}
							onShowAll={onShowAllOverlays}
							onHideAll={onHideAllOverlays}
						/>
					</div>

					<div className="mainViewModulesDivider">
						<span className="mainViewModulesDividerLabel">MODULES</span>
					</div>

					<div className="mainViewModulesGrid">{children}</div>
				</>
			)}
		</main>
	);
}
