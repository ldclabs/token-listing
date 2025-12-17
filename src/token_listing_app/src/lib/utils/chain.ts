export type Chain = { 'Evm': bigint } | { 'Icp': bigint } | { 'Sol': bigint }

export function chainId(chain: Chain): [string, bigint] {
  if ('Icp' in chain) return ['ICP', chain.Icp]
  if ('Sol' in chain) return ['SOL', chain.Sol]
  return ['EVM', chain.Evm]
}

export function chainLabel(chain: Chain): string {
  const [label, id] = chainId(chain)
  switch (label) {
    case 'ICP':
      return 'ICP'
    case 'SOL':
      switch (id) {
        case 0n:
          return 'Solana devnet'
        case 1n:
          return 'Solana'
      }
      break
    case 'EVM':
      switch (id) {
        case 1n:
          return 'Ethereum'
        case 56n:
          return 'BSC Chain'
        case 8453n:
          return 'Base'
        case 84532n:
          return 'Base Sepolia'
      }
      break
  }
  return 'Unknown'
}

export function getTokenUrl(chain: Chain, address: string): string {
  const [label, id] = chainId(chain)
  switch (label) {
    case 'ICP':
      return `https://www.icexplorer.io/token/details/${address}`
    case 'SOL':
      switch (id) {
        case 0n:
          return `https://solscan.io/token/${address}?cluster=devnet`
        case 1n:
          return `https://solscan.io/token/${address}`
      }
      break
    case 'EVM':
      switch (id) {
        case 1n:
          return `https://etherscan.io/token/${address}`
        case 56n:
          return `https://bscscan.com/token/${address}`
        case 8453n:
          return `https://basescan.org/token/${address}`
        case 84532n:
          return `https://sepolia.basescan.org/token/${address}`
      }
      break
  }
  return ''
}

export function getAccountUrl(chain: Chain, address: string): string {
  const [label, id] = chainId(chain)
  switch (label) {
    case 'ICP':
      return `https://www.icexplorer.io/account/details/${address}`
    case 'SOL':
      switch (id) {
        case 0n:
          return `https://solscan.io/address/${address}?cluster=devnet`
        case 1n:
          return `https://solscan.io/address/${address}`
      }
      break
    case 'EVM':
      switch (id) {
        case 1n:
          return `https://etherscan.io/address/${address}`
        case 56n:
          return `https://bscscan.com/address/${address}`
        case 8453n:
          return `https://basescan.org/address/${address}`
        case 84532n:
          return `https://sepolia.basescan.org/address/${address}`
      }
      break
  }
  return ''
}

export function getTxUrl(chain: Chain, tx: string): string {
  const [label, id] = chainId(chain)
  switch (label) {
    case 'ICP':
      return ''
    case 'SOL':
      switch (id) {
        case 0n:
          return `https://solscan.io/tx/${tx}?cluster=devnet`
        case 1n:
          return `https://solscan.io/tx/${tx}`
      }
      break
    case 'EVM':
      switch (id) {
        case 1n:
          return `https://etherscan.io/tx/${tx}`
        case 56n:
          return `https://bscscan.com/tx/${tx}`
        case 8453n:
          return `https://basescan.org/tx/${tx}`
        case 84532n:
          return `https://sepolia.basescan.org/tx/${tx}`
      }
      break
  }
  return ''
}

export function getSwapUrl(name: string): string {
  switch (name) {
    case 'Raydium':
      return `https://raydium.io/`
    case 'KongSwap':
      return `https://kongswap.io/`
  }

  return ''
}
