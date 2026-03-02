import { JSX } from 'react';

import './TitleBar.css';

interface TitleBarProps {
	versionLabel: string | null;
}

export function TitleBar({ versionLabel }: TitleBarProps): JSX.Element {
	return (
		<header className="titleBar">
			<img className="titleBarIcon" src="/icon.png" alt="POE Tauri" />
			<div className="titleBarText">
				<span className="titleBarAppName">POE TAURI</span>
				<span className="titleBarSubtitle">Overlay Dashboard</span>
			</div>
			<div className="titleBarSpacer" />
			{versionLabel !== null ? <span className="titleBarVersion">{versionLabel}</span> : null}
		</header>
	);
}
