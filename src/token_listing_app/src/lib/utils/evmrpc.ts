import { createRequest, JSON_MIME_TYPE } from './fetcher'

export class EvmRpc {
  #providers: string[]
  #endpoint: string
  #contract: string

  constructor(providers: string[], contract: string) {
    this.#providers = providers
    this.#endpoint = providers[0] as string
    this.#contract = contract
  }

  async selectProvider() {
    let selected = await Promise.any(
      this.#providers.map(async (url) => {
        await jsonRPC<string>(url, 'eth_chainId', [])
        return url
      })
    )
    this.#endpoint = selected
  }

  async chainId(): Promise<number> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_chainId', [])) || '0x0'
    return parseInt(rt.slice(2), 16)
  }

  async gasPrice(): Promise<bigint> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_gasPrice', [])) || '0x0'
    return BigInt(rt)
  }

  async maxPriorityFeePerGas(): Promise<bigint> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_maxPriorityFeePerGas', [])) ||
      '0x0'
    return BigInt(rt)
  }

  async gasFeeEstimation(gas: bigint = 54000n): Promise<bigint> {
    const [gasPrice, maxPriorityFeePerGas] = await Promise.all([
      this.gasPrice(),
      this.maxPriorityFeePerGas()
    ])
    return gas * (gasPrice + maxPriorityFeePerGas)
  }

  async blockNumber(): Promise<number> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_blockNumber', [])) || '0x0'
    return parseInt(rt.slice(2), 16)
  }

  async getBalance(address: string): Promise<bigint> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_getBalance', [
        address,
        'latest'
      ])) || '0x0'
    return BigInt(rt)
  }

  async getErc20Balance(address: string): Promise<bigint> {
    const data =
      '0x70a08231000000000000000000000000' +
      address.toLowerCase().replace(/^0x/, '')
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_call', [
        {
          to: this.#contract,
          data: data
        },
        'latest'
      ])) || '0x0'
    return BigInt(rt)
  }

  async getTransactionReceipt(txHash: string): Promise<any> {
    const rt =
      (await jsonRPC<any>(this.#endpoint, 'eth_getTransactionReceipt', [
        txHash
      ])) || null
    return rt
  }

  async sendRawTransaction(signedTx: string): Promise<string> {
    const rt =
      (await jsonRPC<string>(this.#endpoint, 'eth_sendRawTransaction', [
        signedTx
      ])) || '0x'
    return rt
  }
}

export async function jsonRPC<T>(
  url: string,
  method: string,
  params: unknown[] = [],
  headers?: HeadersInit,
  signal?: AbortSignal | null | undefined
): Promise<T | null> {
  const request = createRequest(url, {
    headers: {
      'Content-Type': JSON_MIME_TYPE
    }
  })

  const body = {
    id: 1,
    jsonrpc: '2.0',
    method,
    params
  }

  const res = await request.post<{
    jsonrpc: string
    result?: T
    error?: { code: number; message: string; data?: unknown }
    id: string | number
  }>(url, body, headers, signal)

  if (res.error) {
    const { code, message, data } = res.error
    const error = new Error(
      `JSON-RPC Error ${code}: ${message}${data ? ` - ${JSON.stringify(data)}` : ''}`
    )
    throw error
  }

  return res.result || null
}
