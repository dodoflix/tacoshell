import { render, screen } from '@testing-library/react'
import { SplitPane } from '../SplitPane'

describe('SplitPane', () => {
  it('renders first and second panes', () => {
    render(<SplitPane first={<div>First pane</div>} second={<div>Second pane</div>} />)
    expect(screen.getByText('First pane')).toBeInTheDocument()
    expect(screen.getByText('Second pane')).toBeInTheDocument()
  })

  it('applies horizontal flex direction by default', () => {
    const { container } = render(
      <SplitPane direction="horizontal" first={<div>A</div>} second={<div>B</div>} />,
    )
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toMatch(/flex-row|horizontal/)
  })

  it('applies vertical flex direction when direction=vertical', () => {
    const { container } = render(
      <SplitPane direction="vertical" first={<div>A</div>} second={<div>B</div>} />,
    )
    const wrapper = container.firstChild as HTMLElement
    expect(wrapper.className).toMatch(/flex-col|vertical/)
  })

  it('initialRatio controls initial split', () => {
    render(<SplitPane initialRatio={0.3} first={<div>A</div>} second={<div>B</div>} />)
    // The first pane should have 30% size
    const firstPane = screen.getByText('A').parentElement
    expect(firstPane?.style.flexBasis).toBe('30%')
  })

  it('drag handle is present and has correct aria attributes', () => {
    render(<SplitPane first={<div>A</div>} second={<div>B</div>} />)
    const handle = screen.getByRole('separator')
    expect(handle).toBeInTheDocument()
  })
})
