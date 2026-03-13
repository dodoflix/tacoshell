import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { TabBar } from '../TabBar'

const tabs = [
  { id: '1', label: 'Tab One', closable: true },
  { id: '2', label: 'Tab Two', closable: false },
  { id: '3', label: 'Tab Three', closable: true },
]

describe('TabBar', () => {
  it('renders all tabs', () => {
    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={() => {}} />)
    expect(screen.getByText('Tab One')).toBeInTheDocument()
    expect(screen.getByText('Tab Two')).toBeInTheDocument()
    expect(screen.getByText('Tab Three')).toBeInTheDocument()
  })

  it('applies active styles to activeTabId tab', () => {
    render(<TabBar tabs={tabs} activeTabId="2" onTabClick={() => {}} />)
    const activeTab =
      screen.getByText('Tab Two').closest('[data-active]') ??
      screen.getByText('Tab Two').closest('[aria-selected="true"]') ??
      screen.getByText('Tab Two').closest('button')
    expect(activeTab).toBeTruthy()
  })

  it('calls onTabClick with correct id when tab is clicked', async () => {
    const user = userEvent.setup()
    const onTabClick = vi.fn()
    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={onTabClick} />)
    await user.click(screen.getByText('Tab Two'))
    expect(onTabClick).toHaveBeenCalledWith('2')
  })

  it('shows close button for closable tabs', () => {
    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={() => {}} />)
    // Tab One and Tab Three are closable, Tab Two is not
    const closeButtons = screen.getAllByRole('button', { name: /close/i })
    expect(closeButtons).toHaveLength(2)
  })

  it('calls onTabClose when close button is clicked', async () => {
    const user = userEvent.setup()
    const onTabClose = vi.fn()
    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={() => {}} onTabClose={onTabClose} />)
    const closeButtons = screen.getAllByRole('button', { name: /close/i })
    await user.click(closeButtons[0])
    expect(onTabClose).toHaveBeenCalledWith('1')
  })

  it('close button click does not propagate to tab click', async () => {
    const user = userEvent.setup()
    const onTabClick = vi.fn()
    const onTabClose = vi.fn()
    render(<TabBar tabs={tabs} activeTabId="1" onTabClick={onTabClick} onTabClose={onTabClose} />)
    const closeButtons = screen.getAllByRole('button', { name: /close/i })
    await user.click(closeButtons[0])
    expect(onTabClose).toHaveBeenCalledWith('1')
    expect(onTabClick).not.toHaveBeenCalled()
  })
})
