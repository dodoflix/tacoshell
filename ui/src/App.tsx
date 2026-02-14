import { Sidebar } from "./components/Sidebar";
import { TabBar } from "./components/TabBar";
import { MainContent } from "./components/MainContent";
import "./App.css";

function App() {
  return (
    <div className="app">
      <Sidebar />
      <div className="main-area">
        <TabBar />
        <MainContent />
      </div>
    </div>
  );
}

export default App;
