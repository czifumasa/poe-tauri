import { JSX } from 'react';

import './TitleBar.css';

interface TitleBarProps {
	version: string;
}

export function TitleBar({ version }: TitleBarProps): JSX.Element {
	return (
		<header className="titleBar">
			<img className="titleBarIcon" src="/icon.png" alt="POE Tauri" />
			<div className="titleBarText">
				<span className="titleBarAppName">POE TAURI</span>
				<span className="titleBarSubtitle">Overlay Dashboard</span>
			</div>
			<div className="titleBarSpacer" />
			<span className="titleBarVersion">{version}</span>
		</header>
	);
}
