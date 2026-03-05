import { JSX, type ReactNode } from 'react';

import { SectionDivider } from '../SectionDivider/SectionDivider.tsx';
import { TitleBar } from '../TitleBar/TitleBar.tsx';

import './MainView.css';

interface MainViewProps {
	versionLabel: string | null;
	children: ReactNode;
	settingsContent?: ReactNode;
	onOpenSettings: () => void;
}

function SettingsGearIcon(): JSX.Element {
	return (
		<svg
			width="18"
			height="18"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<circle cx="12" cy="12" r="3" />
			<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
		</svg>
	);
}

export function MainView({ versionLabel, children, settingsContent, onOpenSettings }: MainViewProps): JSX.Element {
	return (
		<main className="mainViewContainer">
			<TitleBar
				leagueName="Settlers"
				leagueDetail="Hardcore · SSF"
				characterName="Exile"
				characterClass="Witch"
				characterLevel={1}
			/>

			{settingsContent !== undefined ? (
				<div className="mainViewSettingsArea">{settingsContent}</div>
			) : (
				<>
					<div className="mainViewDividerWrapper">
						<SectionDivider label="MODULES" />
					</div>

					<div className="mainViewModulesGrid">{children}</div>

					<div className="mainViewFooter">
						{versionLabel !== null ? <span className="mainViewVersionBadge">{versionLabel}</span> : null}
						<button type="button" className="mainViewSettingsButton" onClick={onOpenSettings} title="Settings">
							<SettingsGearIcon />
						</button>
					</div>
				</>
			)}
		</main>
	);
}
