import { JSX, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AscendancyClassEntry } from '../../../types/Guide.ts';
import type { TimerState } from '../../../types/Timer.ts';
import { formatElapsedMs } from '../../../utils/formatTime.ts';
import { SectionDivider } from '../../SectionDivider/SectionDivider.tsx';

import './SaveCampaignRunPage.css';

type LeagueOption = 'Mirage' | 'Standard';

const LEAGUE_OPTIONS: readonly { readonly value: LeagueOption; readonly label: string }[] = [
	{ value: 'Mirage', label: 'Mirage' },
	{ value: 'Standard', label: 'Standard' },
];

interface SaveCampaignRunPageProps {
	timerState: TimerState;
	onBack: () => void;
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

function buildClassOptions(
	entries: readonly AscendancyClassEntry[],
): readonly { readonly value: string; readonly label: string }[] {
	const baseClasses = new Set<string>();
	for (const entry of entries) {
		baseClasses.add(entry.baseClass);
	}

	const options: { value: string; label: string }[] = [];

	for (const base of baseClasses) {
		const capitalized = base.charAt(0).toUpperCase() + base.slice(1);
		options.push({ value: base, label: capitalized });
		for (const entry of entries) {
			if (entry.baseClass === base) {
				const ascCapitalized = entry.ascendancyClass.charAt(0).toUpperCase() + entry.ascendancyClass.slice(1);
				options.push({ value: entry.ascendancyClass, label: `  ${ascCapitalized}` });
			}
		}
	}

	return options;
}

export function SaveCampaignRunPage({ timerState, onBack }: SaveCampaignRunPageProps): JSX.Element {
	const [characterName, setCharacterName] = useState<string>('');
	const [selectedClass, setSelectedClass] = useState<string>('');
	const [league, setLeague] = useState<LeagueOption>('Mirage');
	const [hardcore, setHardcore] = useState<boolean>(false);
	const [ssf, setSsf] = useState<boolean>(false);
	const [privateLeague, setPrivateLeague] = useState<boolean>(false);
	const [runDetails, setRunDetails] = useState<string>('');
	const [classOptions, setClassOptions] = useState<readonly { readonly value: string; readonly label: string }[]>([]);

	useEffect((): void => {
		void invoke<AscendancyClassEntry[]>('get_ascendancy_classes')
			.then((entries) => {
				const options = buildClassOptions(entries);
				setClassOptions(options);
				if (options.length > 0 && selectedClass === '') {
					setSelectedClass(options[0].value);
				}
			})
			.catch((err: unknown) => {
				console.error('Failed to load ascendancy classes:', err);
			});
	}, []);

	const missingFields: string[] = [];
	if (characterName.trim() === '') missingFields.push('character name');
	if (selectedClass === '') missingFields.push('class');
	if (runDetails.trim() === '') missingFields.push('run details');
	const isSaveDisabled = missingFields.length > 0;
	const saveTooltip = isSaveDisabled ? `Missing required fields: ${missingFields.join(', ')}` : undefined;

	return (
		<div className="saveCampaignRunPage">
			<SectionDivider label="SAVE RUN" onBack={onBack} />

			<div className="saveCampaignRunSection">
				<div className="saveCampaignRunSectionTitle">Character</div>
				<div className="saveCampaignRunSectionBody">
					<div className="saveCampaignRunField">
						<span className="saveCampaignRunFieldLabel">Name</span>
						<input
							type="text"
							className="saveCampaignRunInput"
							placeholder="Character name"
							value={characterName}
							onChange={(e) => setCharacterName(e.currentTarget.value)}
						/>
					</div>
					<div className="saveCampaignRunField">
						<span className="saveCampaignRunFieldLabel">Class</span>
						<select
							className="settingsSelect"
							value={selectedClass}
							onChange={(e) => setSelectedClass(e.currentTarget.value)}>
							{classOptions.map((opt) => (
								<option key={opt.value} value={opt.value}>
									{opt.label}
								</option>
							))}
						</select>
					</div>
					<div className="saveCampaignRunFieldFull">
						<span className="saveCampaignRunFieldLabel">Run details</span>
						<input
							type="text"
							className="saveCampaignRunInput"
							placeholder="For example: build, strategy, used gear"
							value={runDetails}
							onChange={(e) => setRunDetails(e.currentTarget.value)}
						/>
					</div>
				</div>
			</div>

			<div className="saveCampaignRunSection">
				<div className="saveCampaignRunSectionTitle">League</div>
				<div className="saveCampaignRunSectionBody">
					<div className="saveCampaignRunField">
						<select
							className="settingsSelect"
							value={league}
							onChange={(e) => setLeague(e.currentTarget.value as LeagueOption)}>
							{LEAGUE_OPTIONS.map((opt) => (
								<option key={opt.value} value={opt.value}>
									{opt.label}
								</option>
							))}
						</select>
					</div>
					<div className="saveCampaignRunCheckboxRow">
						<label className="settingsToggle">
							<input type="checkbox" checked={hardcore} onChange={(e) => setHardcore(e.currentTarget.checked)} />
							<span className="settingsToggleLabel">Hardcore</span>
						</label>
						<label className="settingsToggle">
							<input type="checkbox" checked={ssf} onChange={(e) => setSsf(e.currentTarget.checked)} />
							<span className="settingsToggleLabel">SSF</span>
						</label>
						<label className="settingsToggle">
							<input
								type="checkbox"
								checked={privateLeague}
								onChange={(e) => setPrivateLeague(e.currentTarget.checked)}
							/>
							<span className="settingsToggleLabel">Private</span>
						</label>
					</div>
				</div>
			</div>

			<div className="saveCampaignRunSection">
				<div className="saveCampaignRunSectionTitle">Run</div>
				<div className="saveCampaignRunSplitColumns">
					<div className="saveCampaignRunSplitColumn">
						{timerState.actElapsedMs.slice(0, Math.ceil(timerState.actElapsedMs.length / 2)).map((ms, i) => {
							const isActive = i === timerState.currentActIndex && timerState.status !== 'idle';
							const isCompleted = !isActive && i < timerState.currentActIndex && timerState.status !== 'idle';
							const displayMs = isActive ? timerState.currentActElapsedMs : ms;
							const rowClass = isActive
								? 'saveCampaignRunSplitRow saveCampaignRunSplitRow--active'
								: isCompleted
									? 'saveCampaignRunSplitRow saveCampaignRunSplitRow--completed'
									: 'saveCampaignRunSplitRow';

							return (
								<div key={i} className={rowClass}>
									<span className="saveCampaignRunSplitLabel">Act {i + 1}</span>
									<span className="saveCampaignRunSplitValue">
										{isActive || isCompleted ? formatElapsedMs(displayMs) : '--:--:--'}
									</span>
								</div>
							);
						})}
					</div>
					<div className="saveCampaignRunSplitColumnDivider" />
					<div className="saveCampaignRunSplitColumn">
						{timerState.actElapsedMs.slice(Math.ceil(timerState.actElapsedMs.length / 2)).map((ms, i) => {
							const half = Math.ceil(timerState.actElapsedMs.length / 2);
							const actIndex = half + i;
							const isActive = actIndex === timerState.currentActIndex && timerState.status !== 'idle';
							const isCompleted = !isActive && actIndex < timerState.currentActIndex && timerState.status !== 'idle';
							const displayMs = isActive ? timerState.currentActElapsedMs : ms;
							const rowClass = isActive
								? 'saveCampaignRunSplitRow saveCampaignRunSplitRow--active'
								: isCompleted
									? 'saveCampaignRunSplitRow saveCampaignRunSplitRow--completed'
									: 'saveCampaignRunSplitRow';

							return (
								<div key={actIndex} className={rowClass}>
									<span className="saveCampaignRunSplitLabel">Act {actIndex + 1}</span>
									<span className="saveCampaignRunSplitValue">
										{isActive || isCompleted ? formatElapsedMs(displayMs) : '--:--:--'}
									</span>
								</div>
							);
						})}
					</div>
				</div>
				<div className="saveCampaignRunTotalRow">
					<span className="saveCampaignRunTotalLabel">Total</span>
					<span className="saveCampaignRunTotalValue">{formatElapsedMs(timerState.campaignElapsedMs)}</span>
				</div>
			</div>

			<div className="saveCampaignRunActions">
				<button type="button" className="saveCampaignRunSaveButton" disabled={isSaveDisabled} title={saveTooltip}>
					<SaveIcon />
					Save Run
				</button>
			</div>
		</div>
	);
}
