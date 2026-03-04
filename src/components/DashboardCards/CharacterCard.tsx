import { JSX } from 'react';

import './DashboardCards.css';

interface CharacterCardProps {
	characterName: string;
	characterDetail: string;
	onConfigure: () => void;
}

export function CharacterCard({ characterName, characterDetail, onConfigure }: CharacterCardProps): JSX.Element {
	return (
		<div className="dashboardCard">
			<span className="dashboardCardLabel">Character</span>
			<span className="dashboardCardValue">{characterName}</span>
			<span className="dashboardCardDetail">{characterDetail}</span>
			<button type="button" className="dashboardCardConfigureButton" onClick={onConfigure}>
				Configure
			</button>
		</div>
	);
}
