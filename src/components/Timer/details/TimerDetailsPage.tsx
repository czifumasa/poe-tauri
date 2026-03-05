import { JSX, useState } from 'react';
import type { ActRun, SavedRun, TimerState } from '../../../types/Timer.ts';
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

function buildCompletedActRuns(times: readonly number[]): readonly ActRun[] {
	return times.map((ms, i) => ({ actName: `Act ${i + 1}`, elapsedMs: ms, status: 'completed' as const }));
}

function buildPartialActRuns(times: readonly number[], inProgressIndex: number): readonly ActRun[] {
	return Array.from({ length: ACT_COUNT }, (_, i) => {
		if (i < inProgressIndex) return { actName: `Act ${i + 1}`, elapsedMs: times[i], status: 'completed' as const };
		if (i === inProgressIndex) return { actName: `Act ${i + 1}`, elapsedMs: times[i], status: 'in_progress' as const };
		return { actName: `Act ${i + 1}`, elapsedMs: 0, status: 'pending' as const };
	});
}

const MOCK_SAVED_RUNS: readonly SavedRun[] = [
	{
		id: '1',
		league: 'Mirage',
		hardcore: false,
		ssf: false,
		privateLeague: false,
		character: 'SpeedyExile',
		characterClass: 'Elementalist',
		runDetails: 'SRS Necro league starter, rush acts',
		actRuns: buildCompletedActRuns([
			1_020_000, 960_000, 1_080_000, 1_140_000, 900_000, 1_200_000, 1_320_000, 1_080_000, 1_260_000, 1_440_000,
		]),
		campaignElapsedMs: 11_400_000,
		savedAt: Date.now() - 86_400_000 * 3,
	},
	{
		id: '2',
		league: 'Mirage',
		hardcore: false,
		ssf: false,
		privateLeague: false,
		character: 'TrailRunner',
		characterClass: 'Deadeye',
		runDetails: 'Lightning Arrow practice, leveling uniques',
		actRuns: buildCompletedActRuns([
			1_140_000, 1_080_000, 1_200_000, 1_260_000, 1_020_000, 1_380_000, 1_440_000, 1_200_000, 1_380_000, 1_560_000,
		]),
		campaignElapsedMs: 12_660_000,
		savedAt: Date.now() - 86_400_000 * 5,
	},
	{
		id: '3',
		league: 'Mirage',
		hardcore: true,
		ssf: true,
		privateLeague: false,
		character: 'TankMaster',
		characterClass: 'Juggernaut',
		runDetails: 'RF Jugg, safe pathing, over-leveled zones',
		actRuns: buildCompletedActRuns([
			1_260_000, 1_200_000, 1_320_000, 1_380_000, 1_140_000, 1_500_000, 1_560_000, 1_320_000, 1_500_000, 1_680_000,
		]),
		campaignElapsedMs: 13_860_000,
		savedAt: Date.now() - 86_400_000 * 7,
	},
	{
		id: '4',
		league: 'Mirage',
		hardcore: false,
		ssf: true,
		privateLeague: false,
		character: 'SoloSurvivor',
		characterClass: 'Champion',
		runDetails: 'Steel skills Champion, vendor crafted gear',
		actRuns: buildPartialActRuns(
			[1_380_000, 1_320_000, 1_440_000, 1_500_000, 1_260_000, 1_620_000, 420_000, 0, 0, 0],
			6,
		),
		campaignElapsedMs: 8_940_000,
		savedAt: Date.now() - 86_400_000 * 10,
	},
	{
		id: '5',
		league: 'Standard',
		hardcore: false,
		ssf: false,
		privateLeague: true,
		character: 'ChillWitch',
		characterClass: 'Necromancer',
		runDetails: 'Casual SRS Necro with friends, no rush',
		actRuns: buildPartialActRuns(
			[1_500_000, 1_440_000, 1_560_000, 780_000, 0, 0, 0, 0, 0, 0],
			3,
		),
		campaignElapsedMs: 5_280_000,
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

function actRunRowClass(status: ActRun['status']): string {
	if (status === 'in_progress') return 'timerDetailsSplitRow timerDetailsSplitRow--active';
	if (status === 'completed') return 'timerDetailsSplitRow timerDetailsSplitRow--completed';
	return 'timerDetailsSplitRow';
}

function ActSplitList(props: { actRuns: readonly ActRun[] }): JSX.Element {
	const half = Math.ceil(props.actRuns.length / 2);
	const left = props.actRuns.slice(0, half);
	const right = props.actRuns.slice(half);

	return (
		<div className="timerDetailsSplitColumns">
			<div className="timerDetailsSplitColumn">
				{left.map((act, i) => (
					<div key={i} className={actRunRowClass(act.status)}>
						<span className="timerDetailsSplitLabel">{act.actName}</span>
						<span className="timerDetailsSplitValue">
							{act.status === 'pending' ? '--:--:--' : formatElapsedMs(act.elapsedMs)}
						</span>
					</div>
				))}
			</div>
			<div className="timerDetailsSplitColumnDivider" />
			<div className="timerDetailsSplitColumn">
				{right.map((act, i) => (
					<div key={i + half} className={actRunRowClass(act.status)}>
						<span className="timerDetailsSplitLabel">{act.actName}</span>
						<span className="timerDetailsSplitValue">
							{act.status === 'pending' ? '--:--:--' : formatElapsedMs(act.elapsedMs)}
						</span>
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
					const isCompleted = !isActive && i < timerState.currentActIndex && timerState.status !== 'idle';
					const displayMs = isActive ? timerState.currentActElapsedMs : ms;
					const rowClass = isActive
						? 'timerDetailsSplitRow timerDetailsSplitRow--active'
						: isCompleted
							? 'timerDetailsSplitRow timerDetailsSplitRow--completed'
							: 'timerDetailsSplitRow';

					return (
						<div key={i} className={rowClass}>
							<span className="timerDetailsSplitLabel">Act {i + 1}</span>
							<span className="timerDetailsSplitValue">
								{isActive || isCompleted ? formatElapsedMs(displayMs) : '--:--:--'}
							</span>
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

function formatLeagueDisplay(run: SavedRun): string {
	const tags: string[] = [];
	if (run.hardcore) tags.push('HC');
	if (run.ssf) tags.push('SSF');
	if (run.privateLeague) tags.push('Private');
	if (tags.length === 0) return run.league;
	return `${run.league} (${tags.join(', ')})`;
}

function sortRunsByFilter(runs: readonly SavedRun[], filter: BestRunsFilter): readonly SavedRun[] {
	const filtered = filter === 'campaign'
		? [...runs]
		: runs.filter((r) => r.actRuns[filter]?.status === 'completed');
	return filtered.sort((a, b) => {
		if (filter === 'campaign') {
			return a.campaignElapsedMs - b.campaignElapsedMs;
		}
		const actIndex = filter;
		return (a.actRuns[actIndex]?.elapsedMs ?? 0) - (b.actRuns[actIndex]?.elapsedMs ?? 0);
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
					const timeValue = filter === 'campaign' ? run.campaignElapsedMs : (run.actRuns[filter]?.elapsedMs ?? 0);

					return (
						<div key={run.id} className="timerDetailsRunItem">
							<button
								type="button"
								className={isExpanded ? 'timerDetailsRunRow timerDetailsRunRow--expanded' : 'timerDetailsRunRow'}
								onClick={() => toggleExpand(run.id)}>
								<span className="timerDetailsRunRank">#{rank + 1}</span>
								<div className="timerDetailsRunInfo">
									<span className="timerDetailsRunName">{run.runDetails}</span>
									<span className="timerDetailsRunMeta">
										{formatLeagueDisplay(run)} · {run.character} · {run.characterClass}
									</span>
								</div>
								<span className="timerDetailsRunTime">{formatElapsedMs(timeValue)}</span>
								{filter === 'campaign' && <ChevronIcon expanded={isExpanded} />}
							</button>
							{filter === 'campaign' && isExpanded && (
								<div className="timerDetailsRunExpanded">
									<ActSplitList actRuns={run.actRuns} />
								</div>
							)}
						</div>
					);
				})}
			</div>

			{sorted.length === 0 && <div className="timerDetailsEmpty">No saved runs yet.</div>}
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
									<span className="timerDetailsRunName">{run.runDetails}</span>
									<span className="timerDetailsRunMeta">
										{formatLeagueDisplay(run)} · {run.character} · {run.characterClass}
									</span>
								</div>
								<span className="timerDetailsRunTime">{formatElapsedMs(run.campaignElapsedMs)}</span>
								<ChevronIcon expanded={isExpanded} />
							</button>
							{isExpanded && (
								<div className="timerDetailsRunExpanded">
									<ActSplitList actRuns={run.actRuns} />
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

			{runs.length === 0 && <div className="timerDetailsEmpty">No saved runs.</div>}
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
