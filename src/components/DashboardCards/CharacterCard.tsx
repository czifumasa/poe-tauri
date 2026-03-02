import { JSX } from 'react';

import './DashboardCards.css';

interface CharacterCardProps {
	leagueName: string;
	leagueDetail: string;
	characterName: string;
	characterDetail: string;
	onSync: () => void;
}

export function CharacterCard({
	leagueName,
	leagueDetail,
	characterName,
	characterDetail,
	onSync,
}: CharacterCardProps): JSX.Element {
	return (
		<div className="dashboardCard characterCard">
			<div className="characterCardSection">
				<span className="dashboardCardLabel">League</span>
				<span className="dashboardCardValue">{leagueName}</span>
				<span className="dashboardCardDetail">{leagueDetail}</span>
			</div>
			<div className="characterCardSection">
				<span className="dashboardCardLabel">Character</span>
				<span className="dashboardCardValue">{characterName}</span>
				<span className="dashboardCardDetail">{characterDetail}</span>
			</div>
			<button type="button" className="characterCardSyncButton" onClick={onSync}>
				Sync
			</button>
		</div>
	);
}
