import { Sidebar } from './Sidebar';
import { TabBar } from './TabBar';
import { TitleBar } from './TitleBar';
import { StatusBar } from './StatusBar';
import { ContentArea } from './ContentArea';

export function AppLayout() {
  return (
    <div className="flex h-full w-full flex-col overflow-hidden bg-bg text-fg">
      <TitleBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <TabBar />
          <ContentArea />
        </div>
      </div>
      <StatusBar />
    </div>
  );
}
