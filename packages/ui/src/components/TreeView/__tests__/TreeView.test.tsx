import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { TreeView, type TreeNode } from '../TreeView'

const nodes: TreeNode[] = [
  {
    id: 'root1',
    label: 'Root Node 1',
    children: [
      { id: 'child1', label: 'Child 1' },
      { id: 'child2', label: 'Child 2' },
    ],
  },
  {
    id: 'root2',
    label: 'Root Node 2',
    children: [
      {
        id: 'nested-parent',
        label: 'Nested Parent',
        children: [{ id: 'nested-child', label: 'Nested Child' }],
      },
    ],
  },
  { id: 'leaf', label: 'Leaf Node' },
]

describe('TreeView', () => {
  it('renders root nodes', () => {
    render(<TreeView nodes={nodes} />)
    expect(screen.getByText('Root Node 1')).toBeInTheDocument()
    expect(screen.getByText('Root Node 2')).toBeInTheDocument()
    expect(screen.getByText('Leaf Node')).toBeInTheDocument()
  })

  it('does not render nested children initially', () => {
    render(<TreeView nodes={nodes} />)
    expect(screen.queryByText('Child 1')).not.toBeInTheDocument()
    expect(screen.queryByText('Child 2')).not.toBeInTheDocument()
  })

  it('clicking a parent node toggles expansion', async () => {
    const user = userEvent.setup()
    render(<TreeView nodes={nodes} />)
    await user.click(screen.getByText('Root Node 1'))
    expect(screen.getByText('Child 1')).toBeInTheDocument()
    expect(screen.getByText('Child 2')).toBeInTheDocument()
  })

  it('clicking expanded parent collapses it', async () => {
    const user = userEvent.setup()
    render(<TreeView nodes={nodes} />)
    await user.click(screen.getByText('Root Node 1'))
    expect(screen.getByText('Child 1')).toBeInTheDocument()
    await user.click(screen.getByText('Root Node 1'))
    expect(screen.queryByText('Child 1')).not.toBeInTheDocument()
  })

  it('clicking a leaf node calls onNodeClick', async () => {
    const user = userEvent.setup()
    const onNodeClick = vi.fn()
    render(<TreeView nodes={nodes} onNodeClick={onNodeClick} />)
    await user.click(screen.getByText('Leaf Node'))
    expect(onNodeClick).toHaveBeenCalledWith(
      expect.objectContaining({ id: 'leaf', label: 'Leaf Node' }),
    )
  })

  it('expanded nodes show collapse indicator, collapsed nodes show expand indicator', async () => {
    const user = userEvent.setup()
    render(<TreeView nodes={nodes} />)

    // Before expand: expand indicator
    const expandIndicator = screen.getAllByLabelText(/expand/i)
    expect(expandIndicator.length).toBeGreaterThan(0)

    // After expand: collapse indicator
    await user.click(screen.getByText('Root Node 1'))
    const collapseIndicator = screen.getAllByLabelText(/collapse/i)
    expect(collapseIndicator.length).toBeGreaterThan(0)
  })

  it('leaf nodes have no expand/collapse indicator', () => {
    render(<TreeView nodes={[{ id: 'only-leaf', label: 'Just a leaf' }]} />)
    const indicators = screen.queryAllByLabelText(/expand|collapse/i)
    expect(indicators).toHaveLength(0)
  })
})
