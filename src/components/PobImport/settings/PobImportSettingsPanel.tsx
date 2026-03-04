import { JSX, useCallback, useState } from 'react';
import type { PobSettings } from '../../../types/Settings.ts';

import './PobImportSettingsPanel.css';

type PobImportSettingsPanelProps = {
	pobSettings: PobSettings;
	loading: boolean;
	onAddSlot: (pobCode: string) => Promise<void>;
	onRemoveSlot: (slotIndex: number) => Promise<void>;
	onSetCurrentSlot: (slotIndex: number) => Promise<void>;
};

function PobImportForm(props: { onAddSlot: (pobCode: string) => Promise<void>; disabled: boolean }): JSX.Element {
	const [pobInput, setPobInput] = useState<string>('');
	const [importing, setImporting] = useState<boolean>(false);
	const [importError, setImportError] = useState<string | null>(null);

	const handleImport = useCallback(async (): Promise<void> => {
		const trimmed = pobInput.trim();
		if (trimmed === '') {
			return;
		}
		setImporting(true);
		setImportError(null);
		try {
			await props.onAddSlot(trimmed);
			setPobInput('');
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			setImportError(message);
		} finally {
			setImporting(false);
		}
	}, [pobInput, props]);

	return (
		<div className="pobSettingsGroup">
			<div className="pobSettingsGroupTitle">Import New Build</div>
			<div className="pobSettingsImportRow">
				<input
					type="text"
					className="pobSettingsImportInput"
					placeholder="Paste PoB export code"
					value={pobInput}
					onChange={(event) => setPobInput(event.currentTarget.value)}
					disabled={props.disabled || importing}
				/>
				<button
					type="button"
					className="pobSettingsImportButton"
					onClick={() => void handleImport()}
					disabled={props.disabled || importing || pobInput.trim() === ''}>
					{importing ? 'Importing\u2026' : 'Import'}
				</button>
			</div>
			{importError !== null && <div className="pobSettingsImportError">{importError}</div>}
		</div>
	);
}

export function PobImportSettingsPanel(props: PobImportSettingsPanelProps): JSX.Element {
	const { pobSettings, loading } = props;
	const hasSlots = pobSettings.slots.length > 0;

	return (
		<div className="pobSettingsPanel">
			<div className="pobSettingsPanelDescription">Manage your Path of Building builds.</div>

			<PobImportForm onAddSlot={props.onAddSlot} disabled={loading} />

			<div className="pobSettingsGroup">
				<div className="pobSettingsGroupTitle">Imported Builds</div>
				{hasSlots ? (
					<div className="pobSettingsSlotList">
						{pobSettings.slots.map((slot, index) => {
							const isActive = pobSettings.currentSlotIndex === index;
							const slotClassName = isActive ? 'pobSettingsSlot pobSettingsSlot--active' : 'pobSettingsSlot';

							return (
								<div
									key={index}
									className={slotClassName}
									onClick={() => void props.onSetCurrentSlot(index)}
									role="button"
									tabIndex={0}
									onKeyDown={(event) => {
										if (event.key === 'Enter' || event.key === ' ') {
											void props.onSetCurrentSlot(index);
										}
									}}>
									<div className="pobSettingsSlotIndicator" />
									<div className="pobSettingsSlotInfo">
										<span className="pobSettingsSlotClass">{slot.class}</span>
										<span className="pobSettingsSlotGems">{slot.gemCount} gems</span>
									</div>
									<button
										type="button"
										className="pobSettingsSlotRemoveButton"
										onClick={(event) => {
											event.stopPropagation();
											void props.onRemoveSlot(index);
										}}
										disabled={loading}>
										Remove
									</button>
								</div>
							);
						})}
					</div>
				) : (
					<div className="pobSettingsEmptyMessage">
						No builds imported yet. Paste a PoB export code above to add one.
					</div>
				)}
			</div>
		</div>
	);
}
