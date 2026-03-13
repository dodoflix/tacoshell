import '@testing-library/jest-dom'

// Suppress known false-positive Radix UI accessibility warning in the test environment.
// In happy-dom, Dialog portals render outside the component tree, causing Radix's
// DialogTitle detection to fail even when a DialogTitle IS present in the component.
// Radix logs: "`DialogContent` requires a `DialogTitle` for the component to be accessible…"
// TODO: remove once https://github.com/radix-ui/primitives/issues/1103 is resolved upstream.
const originalConsoleError = console.error.bind(console)
console.error = (...args: unknown[]) => {
  const msg = typeof args[0] === 'string' ? args[0] : ''
  if (msg.startsWith('`DialogContent` requires a `DialogTitle`')) return
  originalConsoleError(...args)
}
