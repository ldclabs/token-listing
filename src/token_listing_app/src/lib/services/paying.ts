import { PAYING_ENDPOINT } from '$lib/constants'
import { createCborRequest } from '$lib/utils/fetcher'
import {
  type PaymentPayload,
  type PaymentRequirements,
  type SettleResponse,
  type VerifyResponse
} from '@ldclabs/1paying-kit'

const api = createCborRequest(PAYING_ENDPOINT)

export interface PaymentRequest<T> {
  paymentRequirements: PaymentRequirements
  paymentPayload: PaymentPayload<T>
  nonce?: string
}

export interface PaymentVerifyResult {
  paymentRequirements: PaymentRequirements
  verifyResponse: VerifyResponse
  nonce?: string
}

export interface PaymentSettleResult {
  paymentRequirements: PaymentRequirements
  settleResponse: SettleResponse
  nonce?: string
}

export async function x402Settle<T>(
  req: PaymentRequest<T>
): Promise<{ result: PaymentSettleResult; signature: string }> {
  return api.post('/x402/settle', req)
}

export async function x402Verify<T>(
  req: PaymentRequest<T>
): Promise<{ result: PaymentVerifyResult; signature: string }> {
  return api.post('/x402/verify', req)
}
