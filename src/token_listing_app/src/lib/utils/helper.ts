import { Principal } from '@dfinity/principal'
import { isAddress } from '@solana/kit'

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

export function pruneCanister(canisterId: string, long?: boolean) {
  if (long ?? globalThis.innerWidth >= 640) return canisterId
  return canisterId.slice(0, 7) + '...' + canisterId.slice(-5)
}

export function pruneAddress(id: string, long?: boolean): string {
  if (long ?? globalThis.innerWidth >= 640) {
    return id.length > 27 ? id.slice(0, 13) + '...' + id.slice(-11) : id
  }
  return id.length > 15 ? id.slice(0, 7) + '...' + id.slice(-5) : id
}

export function validateAddress(chain: string, address: string): boolean {
  switch (chain) {
    case 'ICP':
      try {
        Principal.fromText(address)
        return true
      } catch (_) {}
      return false
    case 'SOL':
      return isAddress(address)
    default:
      return /^0x[a-fA-F0-9]{40}$/.test(address)
  }
}

export function formatTimeAgo(timestamp: number) {
  const delta = Date.now() - new Date(timestamp).getTime()
  const minutes = Math.max(Math.round(delta / (60 * 1000)), 1)
  if (minutes > 60 * 24 * 36) {
    const days = Math.round(minutes / (60 * 24))
    return `${days} days ago`
  } else if (minutes > 60) {
    const hours = Math.round(minutes / 60)
    return `${hours} hours ago`
  }
  return `${minutes} minutes ago`
}

export function formatDatetime(tsMs: number | bigint): string {
  const d = new Date(Number(tsMs))
  return d.toLocaleString()
}

export function parseUnits(input: string, decimals: number): bigint {
  const s = input.trim().replace(/[,\s_']/g, '')
  if (!s) throw new Error('Input is empty')
  if (!/^\d*(?:\.\d*)?$/.test(s)) throw new Error('Invalid amount')
  const [i = '0', f = ''] = s.split('.')
  const intPart = i ? BigInt(i) : 0n
  const frac = f.slice(0, decimals).padEnd(decimals, '0')
  const fracPart = frac ? BigInt(frac) : 0n
  return intPart * pow10(decimals) + fracPart
}

export function pow10(n: number): bigint {
  return 10n ** BigInt(Math.max(0, n))
}
