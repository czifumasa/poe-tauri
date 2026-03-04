import { JSX, useState } from 'react';
import type { SavedRun, TimerState } from '../../../types/Timer.ts';
import { formatElapsedMs } from '../../../utils/formatTime.ts';
import { SectionDivider } from '../../SectionDivider/SectionDivider.tsx';

import './TimerDetailsPage.css';

const ACT_COUNT = 10;

type DetailsTab = 'current' | 'best' | 'manage';

type BestRunsFilter = 'campaign' | number;

const TABS: readonly { readonly id: DetailsTab; readonly label: string }[] = [
	{ id: 'current', label: 'Current Run' },
	{ id: 'best', label: 'Best Runs' },
	{ id: 'manage', label: 'Manage Runs' },
];

const FILTER_OPTIONS: readonly { readonly value: BestRunsFilter; readonly label: string }[] = [
	{ value: 'campaign', label: 'Campaign' },
	...Array.from({ length: ACT_COUNT }, (_, i) => ({ value: i as BestRunsFilter, label: `Act ${i + 1}` })),
];

const MOCK_SAVED_RUNS: readonly SavedRun[] = [
	{
		id: '1',
		name: 'Settlers Day 1',
		league: 'Settlers',
		character: 'SpeedyExile',
		characterClass: 'Witch',
		actElapsedMs: [1_020_000, 960_000, 1_080_000, 1_140_000, 900_000, 1_200_000, 1_320_000, 1_080_000, 1_260_000, 1_440_000],
		campaignElapsedMs: 11_400_000,
		savedAt: Date.now() - 86_400_000 * 3,
	},
	{
		id: '2',
		name: 'Settlers Practice',
		league: 'Settlers',
		character: 'TrailRunner',
		characterClass: 'Ranger',
		actElapsedMs: [1_140_000, 1_080_000, 1_200_000, 1_260_000, 1_020_000, 1_380_000, 1_440_000, 1_200_000, 1_380_000, 1_560_000],
		campaignElapsedMs: 12_660_000,
		savedAt: Date.now() - 86_400_000 * 5,
	},
	{
		id: '3',
		name: 'League Start HC',
		league: 'Settlers',
		character: 'TankMaster',
		characterClass: 'Marauder',
		actElapsedMs: [1_260_000, 1_200_000, 1_320_000, 1_380_000, 1_140_000, 1_500_000, 1_560_000, 1_320_000, 1_500_000, 1_680_000],
		campaignElapsedMs: 13_860_000,
		savedAt: Date.now() - 86_400_000 * 7,
	},
	{
		id: '4',
		name: 'SSF Attempt',
		league: 'Settlers',
		character: 'SoloSurvivor',
		characterClass: 'Duelist',
		actElapsedMs: [1_380_000, 1_320_000, 1_440_000, 1_500_000, 1_260_000, 1_620_000, 1_680_000, 1_440_000, 1_620_000, 1_800_000],
		campaignElapsedMs: 15_060_000,
		savedAt: Date.now() - 86_400_000 * 10,
	},
	{
		id: '5',
		name: 'Casual Run',
		league: 'Settlers',
		character: 'ChillWitch',
		characterClass: 'Witch',
		actElapsedMs: [1_500_000, 1_440_000, 1_560_000, 1_620_000, 1_380_000, 1_740_000, 1_800_000, 1_560_000, 1_740_000, 1_920_000],
		campaignElapsedMs: 16_260_000,
		savedAt: Date.now() - 86_400_000 * 14,
	},
];

interface TimerDetailsPageProps {
	timerState: TimerState;
	onBack: () => void;
	onSaveRun: () => void;
	onResetRun: () => void;
}

function SaveIcon(): JSX.Element {
	return (
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
			<polyline points="17 21 17 13 7 13 7 21" />
			<polyline points="7 3 7 8 15 8" />
		</svg>
	);
}

function ResetIcon(): JSX.Element {
	return (
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<polyline points="23 4 23 10 17 10" />
			<path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
		</svg>
	);
}

function TrashIcon(): JSX.Element {
	return (
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<polyline points="3 6 5 6 21 6" />
			<path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
			<path d="M10 11v6" />
			<path d="M14 11v6" />
			<path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
		</svg>
	);
}

function ExportIcon(): JSX.Element {
	return (
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round">
			<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
			<polyline points="7 10 12 15 17 10" />
			<line x1="12" y1="15" x2="12" y2="3" />
		</svg>
	);
}

function ChevronIcon(props: { expanded: boolean }): JSX.Element {
	return (
		<svg
			width="12"
			height="12"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round"
			className={props.expanded ? 'timerDetailsChevron timerDetailsChevron--expanded' : 'timerDetailsChevron'}>
			<polyline points="6 9 12 15 18 9" />
		</svg>
	);
}

function ActSplitList(props: { actElapsedMs: readonly number[] }): JSX.Element {
	const half = Math.ceil(props.actElapsedMs.length / 2);
	const left = props.actElapsedMs.slice(0, half);
	const right = props.actElapsedMs.slice(half);

	return (
		<div className="timerDetailsSplitColumns">
			<div className="timerDetailsSplitColumn">
				{left.map((ms, i) => (
					<div key={i} className="timerDetailsSplitRow">
						<span className="timerDetailsSplitLabel">Act {i + 1}</span>
						<span className="timerDetailsSplitValue">{formatElapsedMs(ms)}</span>
					</div>
				))}
			</div>
			<div className="timerDetailsSplitColumnDivider" />
			<div className="timerDetailsSplitColumn">
				{right.map((ms, i) => (
					<div key={i + half} className="timerDetailsSplitRow">
						<span className="timerDetailsSplitLabel">Act {i + half + 1}</span>
						<span className="timerDetailsSplitValue">{formatElapsedMs(ms)}</span>
					</div>
				))}
			</div>
		</div>
	);
}

function CurrentRunContent(props: {
	timerState: TimerState;
	onSaveRun: () => void;
	onResetRun: () => void;
}): JSX.Element {
	const { timerState } = props;
	const hasRunData = timerState.status !== 'idle';

	return (
		<div className="timerDetailsSection">
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

			<div className="timerDetailsActions">
				<button
					type="button"
					className="timerDetailsActionButton timerDetailsActionButton--save"
					disabled={!hasRunData}
					onClick={props.onSaveRun}>
					<SaveIcon />
					Save Run
				</button>
				<button
					type="button"
					className="timerDetailsActionButton timerDetailsActionButton--reset"
					disabled={!hasRunData}
					onClick={props.onResetRun}>
					<ResetIcon />
					Reset
				</button>
			</div>
		</div>
	);
}

function sortRunsByFilter(runs: readonly SavedRun[], filter: BestRunsFilter): readonly SavedRun[] {
	return [...runs].sort((a, b) => {
		if (filter === 'campaign') {
			return a.campaignElapsedMs - b.campaignElapsedMs;
		}
		const actIndex = filter;
		return (a.actElapsedMs[actIndex] ?? 0) - (b.actElapsedMs[actIndex] ?? 0);
	});
}

function BestRunsContent(): JSX.Element {
	const [filter, setFilter] = useState<BestRunsFilter>('campaign');
	const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
	const sorted = sortRunsByFilter(MOCK_SAVED_RUNS, filter);

	const toggleExpand = (runId: string): void => {
		setExpandedRunId((prev) => (prev === runId ? null : runId));
	};

	return (
		<div className="timerDetailsSection">
			<div className="timerDetailsFilterBar">
				<select
					id="bestRunsFilter"
					className="settingsSelect"
					value={typeof filter === 'number' ? String(filter) : filter}
					onChange={(e): void => {
						const val = e.target.value;
						setFilter(val === 'campaign' ? 'campaign' : Number(val));
					}}>
					{FILTER_OPTIONS.map((opt) => (
						<option key={String(opt.value)} value={String(opt.value)}>
							{opt.label}
						</option>
					))}
				</select>
			</div>

			<div className="timerDetailsRunList">
				{sorted.map((run, rank) => {
					const isExpanded = expandedRunId === run.id;
					const timeValue =
						filter === 'campaign'
							? run.campaignElapsedMs
							: (run.actElapsedMs[filter] ?? 0);

					return (
						<div key={run.id} className="timerDetailsRunItem">
							<button
								type="button"
								className={isExpanded ? 'timerDetailsRunRow timerDetailsRunRow--expanded' : 'timerDetailsRunRow'}
								onClick={() => toggleExpand(run.id)}>
								<span className="timerDetailsRunRank">#{rank + 1}</span>
								<div className="timerDetailsRunInfo">
									<span className="timerDetailsRunName">{run.name}</span>
									<span className="timerDetailsRunMeta">
										{run.league} · {run.character} · {run.characterClass}
									</span>
								</div>
								<span className="timerDetailsRunTime">{formatElapsedMs(timeValue)}</span>
								{filter === 'campaign' && <ChevronIcon expanded={isExpanded} />}
							</button>
							{filter === 'campaign' && isExpanded && (
								<div className="timerDetailsRunExpanded">
									<ActSplitList actElapsedMs={run.actElapsedMs} />
								</div>
							)}
						</div>
					);
				})}
			</div>

			{sorted.length === 0 && (
				<div className="timerDetailsEmpty">No saved runs yet.</div>
			)}
		</div>
	);
}

function ManageRunsContent(): JSX.Element {
	const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
	const [runs, setRuns] = useState<readonly SavedRun[]>(MOCK_SAVED_RUNS);

	const toggleExpand = (runId: string): void => {
		setExpandedRunId((prev) => (prev === runId ? null : runId));
	};

	const deleteRun = (runId: string): void => {
		setRuns((prev) => prev.filter((r) => r.id !== runId));
		if (expandedRunId === runId) {
			setExpandedRunId(null);
		}
	};

	const exportAllRuns = (): void => {
		// Backend not wired yet
		console.info('Export all runs requested (not implemented).');
	};

	return (
		<div className="timerDetailsSection">
			<div className="timerDetailsManageHeader">
				<span className="timerDetailsSectionTitle">All Runs ({runs.length})</span>
				<button
					type="button"
					className="timerDetailsActionButton timerDetailsActionButton--export"
					onClick={exportAllRuns}>
					<ExportIcon />
					Export All
				</button>
			</div>

			<div className="timerDetailsRunList">
				{runs.map((run) => {
					const isExpanded = expandedRunId === run.id;

					return (
						<div key={run.id} className="timerDetailsRunItem">
							<button
								type="button"
								className={isExpanded ? 'timerDetailsRunRow timerDetailsRunRow--expanded' : 'timerDetailsRunRow'}
								onClick={() => toggleExpand(run.id)}>
								<div className="timerDetailsRunInfo">
									<span className="timerDetailsRunName">{run.name}</span>
									<span className="timerDetailsRunMeta">
										{run.league} · {run.character} · {run.characterClass}
									</span>
								</div>
								<span className="timerDetailsRunTime">{formatElapsedMs(run.campaignElapsedMs)}</span>
								<ChevronIcon expanded={isExpanded} />
							</button>
							{isExpanded && (
								<div className="timerDetailsRunExpanded">
									<ActSplitList actElapsedMs={run.actElapsedMs} />
									<div className="timerDetailsRunExpandedActions">
										<button
											type="button"
											className="timerDetailsActionButton timerDetailsActionButton--delete"
											onClick={() => deleteRun(run.id)}>
											<TrashIcon />
											Delete Run
										</button>
									</div>
								</div>
							)}
						</div>
					);
				})}
			</div>

			{runs.length === 0 && (
				<div className="timerDetailsEmpty">No saved runs.</div>
			)}
		</div>
	);
}

export function TimerDetailsPage({ timerState, onBack, onSaveRun, onResetRun }: TimerDetailsPageProps): JSX.Element {
	const [activeTab, setActiveTab] = useState<DetailsTab>('current');

	return (
		<div className="timerDetailsPage">
			<SectionDivider label="RUN TIMER" onBack={onBack} />

			<div className="timerDetailsTabs">
				{TABS.map((tab) => (
					<button
						key={tab.id}
						type="button"
						className={tab.id === activeTab ? 'timerDetailsTab timerDetailsTab--active' : 'timerDetailsTab'}
						onClick={() => setActiveTab(tab.id)}>
						{tab.label}
					</button>
				))}
			</div>

			<div className="timerDetailsContent">
				{activeTab === 'current' && (
					<CurrentRunContent timerState={timerState} onSaveRun={onSaveRun} onResetRun={onResetRun} />
				)}
				{activeTab === 'best' && <BestRunsContent />}
				{activeTab === 'manage' && <ManageRunsContent />}
			</div>
		</div>
	);
}
