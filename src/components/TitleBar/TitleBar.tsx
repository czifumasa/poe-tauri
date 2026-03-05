import { JSX } from 'react';

import './TitleBar.css';

interface TitleBarProps {
	leagueName: string;
	leagueDetail: string;
	characterName: string;
	characterClass: string;
	characterLevel: number;
}

export function TitleBar({
	leagueName,
	leagueDetail,
	characterName,
	characterClass,
	characterLevel,
}: TitleBarProps): JSX.Element {
	return (
		<header className="titleBar">
			<img className="titleBarIcon" src="/icon.png" alt="POE Tauri" />
			<div className="titleBarText">
				<span className="titleBarAppName">POE TAURI</span>
				<span className="titleBarSubtitle">Overlay Dashboard</span>
			</div>
			<div className="titleBarSpacer" />
			<span className="titleBarCharacterInfo">
				<span className="titleBarCharacterGroup">
					<span className="titleBarCharacterLabel">League</span>
					<span className="titleBarCharacterValues">
						<span className="titleBarCharacterSegment">{leagueName}</span>
						<span className="titleBarCharacterDetail">{leagueDetail}</span>
					</span>
				</span>
				<span className="titleBarCharacterDivider" />
				<span className="titleBarCharacterGroup">
					<span className="titleBarCharacterLabel">Character</span>
					<span className="titleBarCharacterValues">
						<span className="titleBarCharacterSegment">{characterName}</span>
						<span className="titleBarCharacterDetail">
							{characterClass} · Lvl {characterLevel}
						</span>
					</span>
				</span>
			</span>
		</header>
	);
}
