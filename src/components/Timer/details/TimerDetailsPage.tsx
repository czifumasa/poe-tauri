import { JSX, useState } from 'react';
import type { TimerState } from '../../../types/Timer.ts';
import { formatElapsedMs } from '../../../utils/formatTime.ts';

import './TimerDetailsPage.css';

const ACT_COUNT = 10;

type TimerDetailsTab = 'campaign' | number;

type TabDefinition = {
	readonly id: TimerDetailsTab;
	readonly label: string;
};

const TABS: readonly TabDefinition[] = [
	{ id: 'campaign', label: 'Campaign' },
	...Array.from({ length: ACT_COUNT }, (_, i): TabDefinition => ({ id: i, label: `Act ${i + 1}` })),
];

interface TimerDetailsPageProps {
	timerState: TimerState;
	onBack: () => void;
}

function CampaignContent(props: { timerState: TimerState }): JSX.Element {
	const { timerState } = props;

	return (
		<div className="timerDetailsSection">
			<div className="timerDetailsSectionTitle">Act Splits</div>
			<div className="timerDetailsSplitList">
				{timerState.actElapsedMs.map((ms, i) => {
					const isActive = i === timerState.currentActIndex && timerState.status !== 'idle';
					const displayMs = isActive ? timerState.currentActElapsedMs : ms;
					const rowClass = isActive
						? 'timerDetailsSplitRow timerDetailsSplitRow--active'
						: 'timerDetailsSplitRow';

					return (
						<div key={i} className={rowClass}>
							<span className="timerDetailsSplitLabel">Act {i + 1}</span>
							<span className="timerDetailsSplitValue">{formatElapsedMs(displayMs)}</span>
						</div>
					);
				})}
			</div>
			<div className="timerDetailsTotalRow">
				<span className="timerDetailsTotalLabel">Total</span>
				<span className="timerDetailsTotalValue">{formatElapsedMs(timerState.campaignElapsedMs)}</span>
			</div>
		</div>
	);
}

function ActContent(props: { actIndex: number; timerState: TimerState }): JSX.Element {
	const { actIndex, timerState } = props;
	const isActive = actIndex === timerState.currentActIndex && timerState.status !== 'idle';
	const elapsed = isActive ? timerState.currentActElapsedMs : (timerState.actElapsedMs[actIndex] ?? 0);
	const campaignTotal = timerState.campaignElapsedMs;
	const percentage = campaignTotal > 0 ? ((elapsed / campaignTotal) * 100).toFixed(1) : '0.0';

	return (
		<div className="timerDetailsSection">
			<div className="timerDetailsActSummary">
				<span className="timerDetailsActTime">{formatElapsedMs(elapsed)}</span>
				{isActive && <span className="timerDetailsActBadge">In Progress</span>}
			</div>
			<div className="timerDetailsStatRow">
				<span className="timerDetailsStatLabel">Share of campaign</span>
				<span className="timerDetailsStatValue">{percentage}%</span>
			</div>
			<div className="timerDetailsStatRow">
				<span className="timerDetailsStatLabel">Campaign total</span>
				<span className="timerDetailsStatValue">{formatElapsedMs(campaignTotal)}</span>
			</div>
		</div>
	);
}

export function TimerDetailsPage({ timerState, onBack }: TimerDetailsPageProps): JSX.Element {
	const [activeTab, setActiveTab] = useState<TimerDetailsTab>('campaign');

	return (
		<div className="timerDetailsPage">
			<div className="timerDetailsHeader">
				<button type="button" className="timerDetailsBackButton" onClick={onBack}>
					← Back
				</button>
				<span className="timerDetailsTitle">Timer Details</span>
			</div>

			<div className="timerDetailsTabs">
				{TABS.map((tab) => (
					<button
						key={String(tab.id)}
						type="button"
						className={tab.id === activeTab ? 'timerDetailsTab timerDetailsTab--active' : 'timerDetailsTab'}
						onClick={() => setActiveTab(tab.id)}>
						{tab.label}
					</button>
				))}
			</div>

			<div className="timerDetailsContent">
				{activeTab === 'campaign' && <CampaignContent timerState={timerState} />}
				{typeof activeTab === 'number' && <ActContent actIndex={activeTab} timerState={timerState} />}
			</div>
		</div>
	);
}
