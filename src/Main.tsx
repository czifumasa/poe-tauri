import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { OVERLAY_VIEW_QUERY_VALUE, HINT_TOOLTIP_VIEW_QUERY_VALUE } from './constants/WindowIdentifiers.ts';

const viewParam = new URLSearchParams(window.location.search).get('view');
if (viewParam === OVERLAY_VIEW_QUERY_VALUE || viewParam === HINT_TOOLTIP_VIEW_QUERY_VALUE) {
	document.documentElement.dataset.view = viewParam;
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
	<React.StrictMode>
		<App />
	</React.StrictMode>,
);
