import { render, screen } from '@testing-library/react'
import { Panel } from '../Panel'

describe('Panel', () => {
  it('renders children', () => {
    render(
      <Panel>
        <p>Panel content</p>
      </Panel>,
    )
    expect(screen.getByText('Panel content')).toBeInTheDocument()
  })

  it('renders title when provided', () => {
    render(
      <Panel title="My Panel">
        <p>Content</p>
      </Panel>,
    )
    expect(screen.getByText('My Panel')).toBeInTheDocument()
  })

  it('renders actions when provided', () => {
    render(
      <Panel title="Panel" actions={<button>Action</button>}>
        <p>Content</p>
      </Panel>,
    )
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument()
  })

  it('applies custom className', () => {
    const { container } = render(
      <Panel className="custom-class">
        <p>Content</p>
      </Panel>,
    )
    expect(container.firstChild).toHaveClass('custom-class')
  })
})
