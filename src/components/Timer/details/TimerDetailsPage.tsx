import { JSX, useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
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

interface TimerDetailsPageProps {
	timerState: TimerState;
	initialTab?: DetailsTab;
	onBack: () => void;
	onSaveRun: () => void;
	onResetRun: () => void;
	onContinueRun: (runId: string) => void;
	onEditRun: (runId: string) => void;
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

function EditIcon(): JSX.Element {
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
			<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
			<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
		</svg>
	);
}

function PlayIcon(): JSX.Element {
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
			<polygon points="5 3 19 12 5 21 5 3" />
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
	const filtered =
		filter === 'campaign'
			? runs.filter((r) => r.status === 'completed')
			: runs.filter((r) => r.actRuns[filter]?.status === 'completed');
	return [...filtered].sort((a, b) => {
		if (filter === 'campaign') {
			return a.campaignElapsedMs - b.campaignElapsedMs;
		}
		const actIndex = filter;
		return (a.actRuns[actIndex]?.elapsedMs ?? 0) - (b.actRuns[actIndex]?.elapsedMs ?? 0);
	});
}

function BestRunsContent(props: { runs: readonly SavedRun[] }): JSX.Element {
	const [filter, setFilter] = useState<BestRunsFilter>('campaign');
	const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
	const sorted = sortRunsByFilter(props.runs, filter);

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

function runStatusClass(status: SavedRun['status']): string {
	if (status === 'completed') return 'timerDetailsRunStatus timerDetailsRunStatus--completed';
	return 'timerDetailsRunStatus timerDetailsRunStatus--inProgress';
}

function runStatusLabel(run: SavedRun): string {
	if (run.status === 'completed') return 'Completed';
	const currentAct = run.actRuns.find((a) => a.status === 'in_progress');
	if (currentAct !== undefined) return `In progress · ${currentAct.actName}`;
	return 'In progress';
}

type ConfirmationVariant = 'default' | 'delete';

function confirmButtonClass(variant: ConfirmationVariant): string {
	if (variant === 'delete') return 'timerDetailsConfirmButton timerDetailsConfirmButton--delete';
	return 'timerDetailsConfirmButton';
}

function InlineConfirmation(props: {
	message: string;
	variant?: ConfirmationVariant;
	onConfirm: () => void;
	onCancel: () => void;
}): JSX.Element {
	return (
		<div className="timerDetailsConfirmBox">
			<span className="timerDetailsConfirmMessage">{props.message}</span>
			<div className="timerDetailsConfirmActions">
				<button type="button" className={confirmButtonClass(props.variant ?? 'default')} onClick={props.onConfirm}>
					Confirm
				</button>
				<button type="button" className="timerDetailsConfirmCancelButton" onClick={props.onCancel}>
					Cancel
				</button>
			</div>
		</div>
	);
}

function ManageRunExpandedPanel(props: {
	run: SavedRun;
	isConfirmingContinue: boolean;
	isConfirmingDelete: boolean;
	onRequestContinue: () => void;
	onConfirmContinue: () => void;
	onCancelContinue: () => void;
	onEditRun: () => void;
	onRequestDelete: () => void;
	onConfirmDelete: () => void;
	onCancelDelete: () => void;
}): JSX.Element {
	const { run, isConfirmingContinue, isConfirmingDelete } = props;
	const hasActiveConfirmation = isConfirmingContinue || isConfirmingDelete;

	return (
		<div className="timerDetailsRunExpanded">
			<ActSplitList actRuns={run.actRuns} />
			<div className="timerDetailsRunExpandedActions">
				{run.status === 'in_progress' && (
					<button
						type="button"
						className="timerDetailsActionButton timerDetailsActionButton--continue"
						disabled={hasActiveConfirmation}
						onClick={props.onRequestContinue}>
						<PlayIcon />
						Continue Run
					</button>
				)}
				<button
					type="button"
					className="timerDetailsActionButton timerDetailsActionButton--save"
					disabled={hasActiveConfirmation}
					onClick={props.onEditRun}>
					<EditIcon />
					Edit Run
				</button>
				<button
					type="button"
					className="timerDetailsActionButton timerDetailsActionButton--delete"
					disabled={hasActiveConfirmation}
					onClick={props.onRequestDelete}>
					<TrashIcon />
					Delete Run
				</button>
			</div>
			{isConfirmingContinue && (
				<InlineConfirmation
					message="Current run progress will be replaced with this saved run."
					onConfirm={props.onConfirmContinue}
					onCancel={props.onCancelContinue}
				/>
			)}
			{isConfirmingDelete && (
				<InlineConfirmation
					message="This run will be permanently deleted."
					variant="delete"
					onConfirm={props.onConfirmDelete}
					onCancel={props.onCancelDelete}
				/>
			)}
		</div>
	);
}

function ManageRunItem(props: {
	run: SavedRun;
	isExpanded: boolean;
	isConfirmingContinue: boolean;
	isConfirmingDelete: boolean;
	onToggle: () => void;
	onRequestContinue: () => void;
	onConfirmContinue: () => void;
	onCancelContinue: () => void;
	onEditRun: () => void;
	onRequestDelete: () => void;
	onConfirmDelete: () => void;
	onCancelDelete: () => void;
}): JSX.Element {
	const { run, isExpanded } = props;

	return (
		<div className="timerDetailsRunItem">
			<button
				type="button"
				className={isExpanded ? 'timerDetailsRunRow timerDetailsRunRow--expanded' : 'timerDetailsRunRow'}
				onClick={props.onToggle}>
				<div className="timerDetailsRunInfo">
					<span className="timerDetailsRunName">{run.runDetails}</span>
					<span className="timerDetailsRunMeta">
						{formatLeagueDisplay(run)} · {run.character} · {run.characterClass}
					</span>
				</div>
				<span className={runStatusClass(run.status)}>{runStatusLabel(run)}</span>
				<span className="timerDetailsRunTime">{formatElapsedMs(run.campaignElapsedMs)}</span>
				<ChevronIcon expanded={isExpanded} />
			</button>
			{isExpanded && (
				<ManageRunExpandedPanel
					run={run}
					isConfirmingContinue={props.isConfirmingContinue}
					isConfirmingDelete={props.isConfirmingDelete}
					onRequestContinue={props.onRequestContinue}
					onConfirmContinue={props.onConfirmContinue}
					onCancelContinue={props.onCancelContinue}
					onEditRun={props.onEditRun}
					onRequestDelete={props.onRequestDelete}
					onConfirmDelete={props.onConfirmDelete}
					onCancelDelete={props.onCancelDelete}
				/>
			)}
		</div>
	);
}

function ManageRunsContent(props: {
	runs: readonly SavedRun[];
	onDeleteRun: (runId: string) => void;
	onContinueRun: (runId: string) => void;
	onEditRun: (runId: string) => void;
}): JSX.Element {
	const { runs, onDeleteRun, onContinueRun, onEditRun } = props;
	const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
	const [confirmContinueRunId, setConfirmContinueRunId] = useState<string | null>(null);
	const [confirmDeleteRunId, setConfirmDeleteRunId] = useState<string | null>(null);

	const clearConfirmations = (): void => {
		setConfirmContinueRunId(null);
		setConfirmDeleteRunId(null);
	};

	const toggleExpand = (runId: string): void => {
		setExpandedRunId((prev) => (prev === runId ? null : runId));
		clearConfirmations();
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
				{runs.map((run) => (
					<ManageRunItem
						key={run.id}
						run={run}
						isExpanded={expandedRunId === run.id}
						isConfirmingContinue={confirmContinueRunId === run.id}
						isConfirmingDelete={confirmDeleteRunId === run.id}
						onToggle={() => toggleExpand(run.id)}
						onRequestContinue={() => {
							clearConfirmations();
							setConfirmContinueRunId(run.id);
						}}
						onConfirmContinue={() => {
							setConfirmContinueRunId(null);
							onContinueRun(run.id);
						}}
						onCancelContinue={() => setConfirmContinueRunId(null)}
						onEditRun={() => onEditRun(run.id)}
						onRequestDelete={() => {
							clearConfirmations();
							setConfirmDeleteRunId(run.id);
						}}
						onConfirmDelete={() => {
							setConfirmDeleteRunId(null);
							onDeleteRun(run.id);
						}}
						onCancelDelete={() => setConfirmDeleteRunId(null)}
					/>
				))}
			</div>

			{runs.length === 0 && <div className="timerDetailsEmpty">No saved runs.</div>}
		</div>
	);
}

export function TimerDetailsPage({
	timerState,
	initialTab = 'current',
	onBack,
	onSaveRun,
	onResetRun,
	onContinueRun,
	onEditRun,
}: TimerDetailsPageProps): JSX.Element {
	const [activeTab, setActiveTab] = useState<DetailsTab>(initialTab);
	const [savedRuns, setSavedRuns] = useState<readonly SavedRun[]>([]);

	useEffect((): void => {
		void invoke<SavedRun[]>('saved_runs_load')
			.then((runs) => setSavedRuns(runs))
			.catch((err: unknown) => console.error('Failed to load saved runs:', err));
	}, []);

	const handleDeleteRun = useCallback((runId: string): void => {
		void invoke<SavedRun[]>('saved_runs_delete', { runId })
			.then((runs) => setSavedRuns(runs))
			.catch((err: unknown) => console.error('Failed to delete run:', err));
	}, []);

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
				{activeTab === 'best' && <BestRunsContent runs={savedRuns} />}
				{activeTab === 'manage' && (
					<ManageRunsContent
						runs={savedRuns}
						onDeleteRun={handleDeleteRun}
						onContinueRun={onContinueRun}
						onEditRun={onEditRun}
					/>
				)}
			</div>
		</div>
	);
}
