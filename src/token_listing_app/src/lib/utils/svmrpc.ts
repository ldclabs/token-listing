import {
  address,
  createSolanaRpc,
  getAddressEncoder,
  getProgramDerivedAddress,
  mainnet,
  type Address,
  type Base64EncodedWireTransaction,
  type Blockhash,
  type Signature
} from '@solana/kit'

const addressEncoder = getAddressEncoder()

export class SvmRpc {
  #providers: string[]
  #rpc
  #mintAddress: string
  #programId: string

  constructor(providers: string[], mintAddress: string, programId: string) {
    this.#providers = providers
    this.#rpc = createSolanaRpc(mainnet(providers[0] as string))
    this.#mintAddress = mintAddress
    this.#programId = programId
  }

  async selectProvider() {
    let selected = await Promise.any(
      this.#providers.map(async (url) => {
        const rpc = createSolanaRpc(mainnet(url))
        await rpc.getLatestBlockhash().send()
        return rpc
      })
    )
    this.#rpc = selected
  }

  async getLatestBlockhash(): Promise<Blockhash> {
    const { value } = await this.#rpc.getLatestBlockhash().send()
    return value.blockhash
  }

  async getBalance(addr: string): Promise<bigint> {
    try {
      const { value } = await this.#rpc.getBalance(address(addr)).send()
      return BigInt(value)
    } catch (e) {
      console.error('Error fetching SOL balance for', addr, ':', e)
      return 0n
    }
  }

  async getAssociatedTokenAddress(addr: string): Promise<Address> {
    const [pda, _] = await getProgramDerivedAddress({
      programAddress: 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL' as Address,
      seeds: [
        // Owner
        addressEncoder.encode(addr as Address),
        // Token program
        addressEncoder.encode(this.#programId as Address),
        // Mint
        addressEncoder.encode(this.#mintAddress as Address)
      ]
    })
    return pda
  }

  async getSplBalance(addr: string): Promise<bigint> {
    const address = await this.getAssociatedTokenAddress(addr)
    try {
      const { value } = await this.#rpc.getTokenAccountBalance(address).send()
      return BigInt(value.amount)
    } catch (e) {
      console.error('Error fetching SPL balance for', addr, ':', e)
      return 0n
    }
  }

  // 'processed', 'confirmed', 'finalized', ''
  async getTransactionStatus(sig: string): Promise<string> {
    const { value } = await this.#rpc
      .getSignatureStatuses([sig as Signature])
      .send()
    return value?.[0]?.confirmationStatus || ''
  }

  async sendRawTransaction(signedTx: string): Promise<string> {
    const rt = await this.#rpc
      .sendTransaction(signedTx as Base64EncodedWireTransaction, {
        encoding: 'base64'
      })
      .send()
    return rt
  }
}
