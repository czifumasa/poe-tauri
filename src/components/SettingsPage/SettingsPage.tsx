import { JSX, type ReactNode, useCallback, useState } from 'react';

import './SettingsPage.css';

export type SettingsTab = 'global' | 'levelingGuide' | 'pobImport';

type TabDefinition = {
	readonly id: SettingsTab;
	readonly label: string;
};

const TABS: readonly TabDefinition[] = [
	{ id: 'global', label: 'Global' },
	{ id: 'levelingGuide', label: 'Leveling Guide' },
	{ id: 'pobImport', label: 'PoB Import' },
] as const;

interface SettingsPageProps {
	activeTab: SettingsTab;
	onTabChange: (tab: SettingsTab) => void;
	onBack: () => void;
	onResetAppData: () => Promise<void>;
	levelingGuideContent: ReactNode;
	pobImportContent: ReactNode;
}

function ResetAppDataSection(props: { onReset: () => Promise<void> }): JSX.Element {
	const [confirmVisible, setConfirmVisible] = useState<boolean>(false);

	const handleReset = useCallback(async (): Promise<void> => {
		await props.onReset();
		setConfirmVisible(false);
	}, [props]);

	return (
		<div className="settingsPageResetSection">
			<div className="settingsPageResetGroup">
				<button
					type="button"
					className="settingsPageResetButton"
					onClick={() => setConfirmVisible(true)}
					disabled={confirmVisible}>
					Reset App Data
				</button>
				{confirmVisible && (
					<div className="settingsPageResetConfirm">
						<span className="settingsPageResetWarning">
							This resets all settings and clears guide progress. This action cannot be undone.
						</span>
						<div className="settingsPageResetConfirmActions">
							<button type="button" className="settingsPageResetConfirmButton" onClick={() => void handleReset()}>
								Confirm Reset
							</button>
							<button type="button" className="settingsPageResetCancelButton" onClick={() => setConfirmVisible(false)}>
								Cancel
							</button>
						</div>
					</div>
				)}
			</div>
		</div>
	);
}

export function SettingsPage({
	activeTab,
	onTabChange,
	onBack,
	onResetAppData,
	levelingGuideContent,
	pobImportContent,
}: SettingsPageProps): JSX.Element {
	return (
		<div className="settingsPage">
			<div className="settingsPageHeader">
				<button type="button" className="settingsPageBackButton" onClick={onBack}>
					← Back
				</button>
				<span className="settingsPageTitle">Settings</span>
			</div>

			<div className="settingsPageTabs">
				{TABS.map((tab) => (
					<button
						key={tab.id}
						type="button"
						className={tab.id === activeTab ? 'settingsPageTab settingsPageTab--active' : 'settingsPageTab'}
						onClick={() => onTabChange(tab.id)}>
						{tab.label}
					</button>
				))}
			</div>

			<div className="settingsPageContent">
				{activeTab === 'global' && <ResetAppDataSection onReset={onResetAppData} />}
				{activeTab === 'levelingGuide' && levelingGuideContent}
				{activeTab === 'pobImport' && pobImportContent}
			</div>
		</div>
	);
}
