/**
 * Generates a cryptographically random hex string of the given length.
 *
 * Uses ceil(length/2) bytes so that hex encoding yields exactly `length` hex
 * characters, preserving the full entropy of every source byte.
 */
export function generateRandom(length: number): string {
  const array = new Uint8Array(Math.ceil(length / 2))
  crypto.getRandomValues(array)
  return Array.from(array, (byte) => byte.toString(16).padStart(2, '0'))
    .join('')
    .slice(0, length)
}

/**
 * Derives a PKCE S256 code challenge from the given plain-text verifier.
 * Returns a base64url-encoded SHA-256 digest.
 */
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(digest)
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
}
