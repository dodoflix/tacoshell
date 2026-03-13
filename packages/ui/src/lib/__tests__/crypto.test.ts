import { describe, it, expect } from 'vitest'
import { generateRandom, generateCodeChallenge } from '../crypto'

describe('generateRandom', () => {
  it('returns a string of exactly the requested length', () => {
    for (const len of [16, 32, 43, 64]) {
      expect(generateRandom(len)).toHaveLength(len)
    }
  })

  it('returns only hex characters', () => {
    const result = generateRandom(64)
    expect(result).toMatch(/^[0-9a-f]+$/)
  })

  it('returns different values on successive calls', () => {
    const a = generateRandom(32)
    const b = generateRandom(32)
    expect(a).not.toBe(b)
  })
})

describe('generateCodeChallenge', () => {
  it('returns a non-empty base64url string', async () => {
    const challenge = await generateCodeChallenge('test-verifier')
    expect(challenge.length).toBeGreaterThan(0)
    // base64url must not contain +, /, or = padding
    expect(challenge).not.toMatch(/[+/=]/)
    expect(challenge).toMatch(/^[A-Za-z0-9\-_]+$/)
  })

  it('is deterministic for the same verifier', async () => {
    const v = 'deterministic-verifier-value'
    const a = await generateCodeChallenge(v)
    const b = await generateCodeChallenge(v)
    expect(a).toBe(b)
  })

  it('produces different challenges for different verifiers', async () => {
    const a = await generateCodeChallenge('verifier-one')
    const b = await generateCodeChallenge('verifier-two')
    expect(a).not.toBe(b)
  })

  it('meets RFC 7636 length requirement (43–128 chars for SHA-256)', async () => {
    // SHA-256 digest = 32 bytes → ceil(32*4/3) = 43 base64url chars (no padding)
    const challenge = await generateCodeChallenge('any-verifier')
    expect(challenge.length).toBeGreaterThanOrEqual(43)
    expect(challenge.length).toBeLessThanOrEqual(128)
  })
})
