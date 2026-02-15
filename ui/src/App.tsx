import { useEffect } from "react";
import { useAppStore } from "./stores/appStore";
import { Sidebar } from "./components/Sidebar";
import { TabBar } from "./components/TabBar";
import { ViewManager } from "./components/ViewManager";

function App() {
  const { theme } = useAppStore();

  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [theme]);

  return (
    <div className="flex h-screen overflow-hidden bg-background-dark text-slate-200 font-display">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
        <TabBar />
        <ViewManager />
      </div>
    </div>
  );
}

export default App;
