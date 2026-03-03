import { JSX } from 'react';
import type { PobSettings } from '../../../types/Settings.ts';
import { ModuleSnippet } from '../../ModuleSnippet/ModuleSnippet.tsx';

import './PobImportDashboardSnippet.css';

type PobImportDashboardSnippetProps = {
	pobSettings: PobSettings;
	onOpenSettings: () => void;
};

const DESCRIPTION = 'Import and manage Path of Building builds.';

function PobSummaryBody(props: { pobSettings: PobSettings }): JSX.Element {
	const { pobSettings } = props;
	const currentSlot =
		pobSettings.currentSlotIndex !== null ? (pobSettings.slots[pobSettings.currentSlotIndex] ?? null) : null;

	if (currentSlot === null) {
		return <></>;
	}

	const slotCountLabel =
		pobSettings.slots.length === 1 ? '1 build imported' : `${pobSettings.slots.length} builds imported`;

	return (
		<div className="pobSnippetSummary">
			<div className="pobSnippetSummaryRow">
				<span className="pobSnippetClassLabel">{currentSlot.class}</span>
				<span className="pobSnippetGemLabel">{currentSlot.gemCount} gems</span>
			</div>
			<span className="pobSnippetSlotCount">{slotCountLabel}</span>
		</div>
	);
}

export function PobImportDashboardSnippet(props: PobImportDashboardSnippetProps): JSX.Element {
	const { pobSettings } = props;
	const hasSlots = pobSettings.slots.length > 0;

	const actionLabel = hasSlots ? 'MANAGE BUILDS' : 'IMPORT BUILD';

	return (
		<ModuleSnippet
			title="Path of Building"
			description={DESCRIPTION}
			active={hasSlots}
			action={{ type: 'primary', label: actionLabel, onClick: props.onOpenSettings }}>
			{hasSlots && <PobSummaryBody pobSettings={pobSettings} />}
		</ModuleSnippet>
	);
}
