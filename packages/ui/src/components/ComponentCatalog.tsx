import { useState } from 'react'
import { Button } from './Button/Button'
import { Input } from './Input/Input'
import { Modal } from './Modal/Modal'
import { Panel } from './Panel/Panel'
import { SplitPane } from './SplitPane/SplitPane'
import { TabBar } from './TabBar/TabBar'
import { StatusBadge } from './StatusBadge/StatusBadge'
import { Sidebar } from './Sidebar/Sidebar'
import { TreeView, type TreeNode } from './TreeView/TreeView'

const sampleTabs = [
  { id: '1', label: 'server-01', closable: true },
  { id: '2', label: 'server-02', closable: true },
  { id: '3', label: 'local', closable: false },
]

const sampleNodes: TreeNode[] = [
  {
    id: 'connections',
    label: 'Connections',
    children: [
      { id: 'ssh1', label: 'Production Server' },
      { id: 'ssh2', label: 'Staging Server' },
      {
        id: 'k8s',
        label: 'Kubernetes',
        children: [
          { id: 'pod1', label: 'web-app-pod-1' },
          { id: 'pod2', label: 'api-pod-1' },
        ],
      },
    ],
  },
  { id: 'local', label: 'Local Machine' },
]

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="mb-10">
      <h2 className="mb-4 border-b pb-2 text-xl font-semibold text-gray-800">{title}</h2>
      <div className="flex flex-wrap gap-4">{children}</div>
    </section>
  )
}

export function ComponentCatalog() {
  const [modalOpen, setModalOpen] = useState(false)
  const [activeTab, setActiveTab] = useState('1')
  const [tabs, setTabs] = useState(sampleTabs)

  const handleTabClose = (id: string) => {
    setTabs((prev) => prev.filter((t) => t.id !== id))
  }

  return (
    <div className="min-h-screen bg-gray-100 p-8">
      <h1 className="mb-8 text-3xl font-bold text-gray-900">Tacoshell Component Catalog</h1>

      <Section title="Button">
        <Button variant="primary">Primary</Button>
        <Button variant="secondary">Secondary</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="icon" aria-label="Close">
          ✕
        </Button>
        <Button loading>Loading...</Button>
        <Button disabled>Disabled</Button>
      </Section>

      <Section title="Input">
        <div className="w-64">
          <Input label="Username" placeholder="Enter username" />
        </div>
        <div className="w-64">
          <Input variant="password" label="Password" placeholder="Enter password" />
        </div>
        <div className="w-64">
          <Input variant="search" placeholder="Search..." />
        </div>
        <div className="w-64">
          <Input label="With Error" error="This field is required" />
        </div>
      </Section>

      <Section title="Modal">
        <Button onClick={() => setModalOpen(true)}>Open Modal</Button>
        <Modal open={modalOpen} onClose={() => setModalOpen(false)} title="Example Modal">
          <p className="text-gray-600">
            This is modal content. Press Escape or click outside to close.
          </p>
          <div className="mt-4 flex justify-end gap-2">
            <Button variant="ghost" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={() => setModalOpen(false)}>Confirm</Button>
          </div>
        </Modal>
      </Section>

      <Section title="Panel">
        <div className="w-80">
          <Panel title="Simple Panel" actions={<Button variant="ghost">Edit</Button>}>
            <p className="text-sm text-gray-600">Panel body content goes here.</p>
          </Panel>
        </div>
        <div className="w-80">
          <Panel>
            <p className="text-sm text-gray-600">Panel without title.</p>
          </Panel>
        </div>
      </Section>

      <Section title="SplitPane">
        <div className="h-48 w-full overflow-hidden rounded border">
          <SplitPane
            direction="horizontal"
            first={<div className="h-full bg-blue-50 p-4">First Pane</div>}
            second={<div className="h-full bg-green-50 p-4">Second Pane</div>}
          />
        </div>
        <div className="h-48 w-96 overflow-hidden rounded border">
          <SplitPane
            direction="vertical"
            initialRatio={0.3}
            first={<div className="h-full bg-purple-50 p-4">Top Pane</div>}
            second={<div className="h-full bg-yellow-50 p-4">Bottom Pane</div>}
          />
        </div>
      </Section>

      <Section title="TabBar">
        <div className="w-full">
          <TabBar
            tabs={tabs}
            activeTabId={activeTab}
            onTabClick={setActiveTab}
            onTabClose={handleTabClose}
          />
          <div className="rounded-b border border-t-0 bg-white p-4 text-sm text-gray-600">
            Active tab: {tabs.find((t) => t.id === activeTab)?.label ?? 'none'}
          </div>
        </div>
      </Section>

      <Section title="StatusBadge">
        <StatusBadge status="connected" label="Production" />
        <StatusBadge status="disconnected" label="Offline Server" />
        <StatusBadge status="loading" label="Connecting..." />
        <StatusBadge status="error" label="Connection Failed" />
        <StatusBadge status="connected" />
      </Section>

      <Section title="Sidebar">
        <div className="h-64 w-64 overflow-hidden rounded border">
          <Sidebar
            header={<span className="text-sm font-semibold">Connections</span>}
            footer={<span className="text-xs text-gray-500">v0.1.0</span>}
          >
            <p className="text-sm text-gray-600">Sidebar body content</p>
          </Sidebar>
        </div>
      </Section>

      <Section title="TreeView">
        <div className="w-64 rounded border bg-white p-2">
          <TreeView
            nodes={sampleNodes}
            onNodeClick={(_node) => {
              /* visual catalog — no-op */
            }}
          />
        </div>
      </Section>
    </div>
  )
}
