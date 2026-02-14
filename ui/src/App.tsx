import { GlobalSidebar } from "./components/GlobalSidebar";
import { Sidebar } from "./components/Sidebar";
import { TabBar } from "./components/TabBar";
import { MainContent } from "./components/MainContent";

function App() {
  return (
    <div className="flex h-screen overflow-hidden bg-background-dark text-slate-200 font-display">
      <GlobalSidebar />
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0 h-full overflow-hidden">
        <TabBar />
        <MainContent />
      </div>
    </div>
  );
}

export default App;
