import { JSX, type ReactNode } from 'react';

import './ModuleSnippet.css';

type ModuleSnippetAction =
	| { type: 'primary'; label: string; onClick: () => void; disabled?: boolean }
	| { type: 'comingSoon' };

interface ModuleSnippetProps {
	title: string;
	description: string;
	active?: boolean;
	disabled?: boolean;
	showButton?: { label: string; onClick: () => void; disabled?: boolean };
	action: ModuleSnippetAction;
	onSettingsClick?: () => void;
	settingsDisabled?: boolean;
	children?: ReactNode;
}

function SettingsIcon(): JSX.Element {
	return (
		<svg
			width="16"
			height="16"
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

function renderFooterAction(action: ModuleSnippetAction): JSX.Element {
	if (action.type === 'comingSoon') {
		return (
			<button type="button" className="moduleSnippetComingSoonButton" disabled>
				COMING SOON
			</button>
		);
	}

	return (
		<button type="button" className="moduleSnippetActionButton" onClick={action.onClick} disabled={action.disabled}>
			{action.label}
		</button>
	);
}

export function ModuleSnippet(props: ModuleSnippetProps): JSX.Element {
	const classNames = [
		'moduleSnippet',
		props.active === true ? 'moduleSnippet--active' : '',
		props.disabled === true ? 'moduleSnippet--disabled' : '',
	]
		.filter(Boolean)
		.join(' ');

	return (
		<div className={classNames}>
			<div className="moduleSnippetHeader">
				<span className="moduleSnippetTitle">{props.title}</span>
				{props.showButton !== undefined && (
					<button
						type="button"
						className="moduleSnippetShowButton"
						onClick={props.showButton.onClick}
						disabled={props.showButton.disabled}>
						{props.showButton.label}
					</button>
				)}
			</div>

			<div className="moduleSnippetDescription">{props.description}</div>

			<div className="moduleSnippetBody">{props.children}</div>

			<div className="moduleSnippetFooter">
				{renderFooterAction(props.action)}
				{props.onSettingsClick !== undefined && (
					<button
						type="button"
						className="moduleSnippetSettingsButton"
						onClick={props.onSettingsClick}
						disabled={props.settingsDisabled}>
						<SettingsIcon />
					</button>
				)}
			</div>
		</div>
	);
}
