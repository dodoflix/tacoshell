// Main content area with tab content

import { useAppStore } from '../stores/appStore';
import { SplitPaneLayout } from './SplitPaneLayout';

export function MainContent() {
  const { tabs } = useAppStore();

  return <SplitPaneLayout tabs={tabs} />;
}
