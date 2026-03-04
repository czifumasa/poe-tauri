import { JSX } from 'react';

import './DashboardCards.css';

interface LeagueCardProps {
	leagueName: string;
	leagueDetail: string;
	onConfigure: () => void;
}

export function LeagueCard({ leagueName, leagueDetail, onConfigure }: LeagueCardProps): JSX.Element {
	return (
		<div className="dashboardCard">
			<span className="dashboardCardLabel">League</span>
			<span className="dashboardCardValue">{leagueName}</span>
			<span className="dashboardCardDetail">{leagueDetail}</span>
			<button type="button" className="dashboardCardConfigureButton" onClick={onConfigure}>
				Configure
			</button>
		</div>
	);
}
