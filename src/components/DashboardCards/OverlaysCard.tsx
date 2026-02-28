import { JSX } from 'react';

import './DashboardCards.css';

interface OverlaysCardProps {
	allVisible: boolean;
	onShowAll: () => void;
	onHideAll: () => void;
}

export function OverlaysCard({ allVisible, onShowAll, onHideAll }: OverlaysCardProps): JSX.Element {
	const dotClass = allVisible ? 'overlaysStatusDot overlaysStatusDot--visible' : 'overlaysStatusDot overlaysStatusDot--hidden';
	const statusText = allVisible ? 'All Visible' : 'Hidden';

	return (
		<div className="dashboardCard overlaysCard">
			<span className="dashboardCardLabel">Overlays</span>
			<div className="overlaysStatusRow">
				<span className={dotClass} />
				<span className="overlaysStatusText">{statusText}</span>
			</div>
			<div className="overlaysButtonRow">
				<button type="button" className="overlaysShowAllButton" onClick={onShowAll}>
					Show All
				</button>
				<button type="button" className="overlaysHideAllButton" onClick={onHideAll}>
					Hide All
				</button>
			</div>
		</div>
	);
}
