import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { createRef } from 'react'
import { Input } from '../Input'

describe('Input', () => {
  it('renders text input by default', () => {
    render(<Input />)
    expect(screen.getByRole('textbox')).toBeInTheDocument()
  })

  it('renders password type for password variant', () => {
    render(<Input variant="password" />)
    const input = document.querySelector('input[type="password"]')
    expect(input).toBeInTheDocument()
  })

  it('clicking toggle reveals password', async () => {
    const user = userEvent.setup()
    render(<Input variant="password" />)
    const toggle = screen.getByRole('button', { name: /show|reveal|toggle/i })
    await user.click(toggle)
    const input = document.querySelector('input[type="text"]')
    expect(input).toBeInTheDocument()
  })

  it('clicking toggle again hides password', async () => {
    const user = userEvent.setup()
    render(<Input variant="password" />)
    const toggle = screen.getByRole('button', { name: /show|reveal|toggle/i })
    await user.click(toggle)
    await user.click(toggle)
    const input = document.querySelector('input[type="password"]')
    expect(input).toBeInTheDocument()
  })

  it('renders search icon for search variant', () => {
    render(<Input variant="search" />)
    const icon = document.querySelector('[data-testid="search-icon"], svg, [aria-hidden="true"]')
    expect(icon).toBeInTheDocument()
  })

  it('shows error message when error prop is provided', () => {
    render(<Input error="This field is required" />)
    expect(screen.getByText('This field is required')).toBeInTheDocument()
  })

  it('shows label when label prop is provided', () => {
    render(<Input label="Username" />)
    expect(screen.getByLabelText('Username')).toBeInTheDocument()
  })

  it('forwards ref', () => {
    const ref = createRef<HTMLInputElement>()
    render(<Input ref={ref} />)
    expect(ref.current).toBeInstanceOf(HTMLInputElement)
  })
})
