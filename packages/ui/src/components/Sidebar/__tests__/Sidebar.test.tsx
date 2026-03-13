import { render, screen } from '@testing-library/react'
import { Sidebar } from '../Sidebar'

describe('Sidebar', () => {
  it('renders children', () => {
    render(
      <Sidebar>
        <p>Sidebar content</p>
      </Sidebar>,
    )
    expect(screen.getByText('Sidebar content')).toBeInTheDocument()
  })

  it('renders header when provided', () => {
    render(
      <Sidebar header={<div>Header content</div>}>
        <p>Content</p>
      </Sidebar>,
    )
    expect(screen.getByText('Header content')).toBeInTheDocument()
  })

  it('renders footer when provided', () => {
    render(
      <Sidebar footer={<div>Footer content</div>}>
        <p>Content</p>
      </Sidebar>,
    )
    expect(screen.getByText('Footer content')).toBeInTheDocument()
  })

  it('applies custom className', () => {
    const { container } = render(
      <Sidebar className="custom-sidebar">
        <p>Content</p>
      </Sidebar>,
    )
    expect(container.firstChild).toHaveClass('custom-sidebar')
  })
})
