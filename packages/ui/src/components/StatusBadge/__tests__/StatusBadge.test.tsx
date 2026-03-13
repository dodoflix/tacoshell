import { render, screen } from '@testing-library/react'
import { StatusBadge } from '../StatusBadge'

describe('StatusBadge', () => {
  it('renders connected status with correct color class (green)', () => {
    const { container } = render(<StatusBadge status="connected" />)
    const indicator = container.querySelector('[class*="green"], [class*="connected"]')
    expect(indicator).toBeInTheDocument()
  })

  it('renders disconnected status with correct color class (gray)', () => {
    const { container } = render(<StatusBadge status="disconnected" />)
    const indicator = container.querySelector(
      '[class*="gray"], [class*="disconnected"], [class*="slate"]',
    )
    expect(indicator).toBeInTheDocument()
  })

  it('renders loading status with animation class', () => {
    const { container } = render(<StatusBadge status="loading" />)
    const indicator = container.querySelector(
      '[class*="animate"], [class*="loading"], [class*="pulse"]',
    )
    expect(indicator).toBeInTheDocument()
  })

  it('renders error status with correct color class (red)', () => {
    const { container } = render(<StatusBadge status="error" />)
    const indicator = container.querySelector('[class*="red"], [class*="error"]')
    expect(indicator).toBeInTheDocument()
  })

  it('renders label when provided', () => {
    render(<StatusBadge status="connected" label="SSH Server" />)
    expect(screen.getByText('SSH Server')).toBeInTheDocument()
  })

  it('has correct aria-label for screen readers', () => {
    render(<StatusBadge status="connected" />)
    const el = screen.getByRole('status')
    expect(el).toHaveAttribute('aria-label')
  })
})
