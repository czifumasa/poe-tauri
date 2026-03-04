import { JSX } from 'react';

import './SectionDivider.css';

interface SectionDividerProps {
	readonly label: string;
	readonly onBack?: () => void;
}

export function SectionDivider({ label, onBack }: SectionDividerProps): JSX.Element {
	return (
		<div className="sectionDivider">
			<div className="sectionDividerRow">
				<div className="sectionDividerLine" />
				<span className="sectionDividerLabel">{label}</span>
				<div className="sectionDividerLine" />
			</div>
			{onBack !== undefined && (
				<button type="button" className="sectionDividerBackButton" onClick={onBack}>
					← Back
				</button>
			)}
		</div>
	);
}
