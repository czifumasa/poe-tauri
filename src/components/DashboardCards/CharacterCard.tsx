import { JSX } from 'react';

import './DashboardCards.css';

interface CharacterCardProps {
	leagueName: string;
	leagueDetail: string;
	characterName: string;
	characterClass: string;
	characterLevel: number;
}

export function CharacterCard({
	leagueName,
	leagueDetail,
	characterName,
	characterClass,
	characterLevel,
}: CharacterCardProps): JSX.Element {
	return (
		<div className="dashboardCard profileCard">
			<div className="profileCardSection">
				<span className="dashboardCardLabel">League</span>
				<span className="profileCardRow">
					<span className="dashboardCardValue">{leagueName}</span>
					<span className="dashboardCardDetail">{leagueDetail}</span>
				</span>
			</div>
			<div className="profileCardSection">
				<span className="dashboardCardLabel">Character</span>
				<span className="profileCardRow">
					<span className="dashboardCardValue">{characterName}</span>
					<span className="dashboardCardDetail">{characterClass} · Lvl {characterLevel}</span>
				</span>
			</div>
		</div>
	);
}
