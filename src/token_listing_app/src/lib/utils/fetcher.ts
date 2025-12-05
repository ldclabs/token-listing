import { sha3_256 } from '@noble/hashes/sha3'
import { decode, encode, rfc8949EncodeOptions } from 'cborg'
import { joinURL, type URLSearchParamsInit } from './url'

export const CBOR_MIME_TYPE = 'application/cbor'
export const JSON_MIME_TYPE = 'application/json'

const EMPTY_DIGEST = sha3_256('https://TokenList.ing')

export interface Signer {
  // return token: 'ICP base64urlxxxxxx'
  signHash: (hash: Uint8Array) => Promise<string | null>
}

export enum RequestMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  PATCH = 'PATCH',
  DELETE = 'DELETE'
}

export interface HTTPError {
  code: number
  message: string
  data?: any
  requestId?: string | null
}

export interface ErrorResponse {
  error: HTTPError
}

export interface SuccessResponse<T> {
  result: T
}

export function createRequest(
  baseURL: string,
  defaultOptions: RequestInit = {},
  signer?: Signer
) {
  const request = async <T>(
    path: string,
    params?: URLSearchParamsInit,
    options?: RequestInit
  ) => {
    const url =
      path.startsWith('http://') || path.startsWith('https://')
        ? joinURL(path, '', params)
        : joinURL(baseURL, path, params)
    const headers = new Headers(defaultOptions.headers)
    new Headers(options?.headers).forEach((value, key) =>
      headers.set(key, value)
    )
    if (!headers.has('Accept')) headers.set('Accept', JSON_MIME_TYPE)

    if (options) {
      options.mode = 'cors'

      if (signer) {
        let hash = EMPTY_DIGEST
        if (typeof options.body === 'string') {
          hash = sha3_256(new TextEncoder().encode(options.body))
        }
        const token = await signer.signHash(hash)
        if (token) {
          headers.set('Authorization', token)
        }
      }
    }

    const resp = await fetch(url, { ...defaultOptions, ...options, headers })
    const body: T = await readResponseBody(resp)
    if (resp.ok) {
      return body as T
    } else {
      const requestId = resp.headers.get('X-Request-Id')
      throw createHTTPError(resp.status, body, requestId)
    }
  }

  request.defaultOptions = Object.freeze(defaultOptions)
  request.get = <T>(
    path: string,
    params?: URLSearchParamsInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.GET,
      signal
    })
  }
  request.post = <T>(
    path: string,
    body?: object,
    headers?: HeadersInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, undefined, {
      method: RequestMethod.POST,
      body: body ? JSON.stringify(body) : null,
      headers: { ...headers, 'Content-Type': JSON_MIME_TYPE },
      signal
    })
  }
  request.put = <T>(
    path: string,
    params?: URLSearchParamsInit,
    body?: object,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.PUT,
      body: body ? JSON.stringify(body) : null,
      headers: { 'Content-Type': JSON_MIME_TYPE },
      signal
    })
  }
  request.patch = <T>(
    path: string,
    params?: URLSearchParamsInit,
    body?: object,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.PATCH,
      body: body ? JSON.stringify(body) : null,
      headers: { 'Content-Type': JSON_MIME_TYPE },
      signal
    })
  }
  request.delete = <T>(
    path: string,
    params?: URLSearchParamsInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.DELETE,
      signal
    })
  }
  return request
}

export function createCborRequest(
  baseURL: string,
  defaultOptions: RequestInit = {},
  signer?: Signer
) {
  const request = async <T>(
    path: string,
    params?: URLSearchParamsInit,
    options?: RequestInit
  ) => {
    const url =
      path.startsWith('http://') || path.startsWith('https://')
        ? joinURL(path, '', params)
        : joinURL(baseURL, path, params)

    const headers = new Headers(defaultOptions.headers)
    new Headers(options?.headers).forEach((value, key) =>
      headers.set(key, value)
    )
    if (!headers.has('Accept')) headers.set('Accept', CBOR_MIME_TYPE)

    if (options) {
      options.mode = 'cors'

      if (signer) {
        let hash = EMPTY_DIGEST
        if (options.body instanceof Uint8Array) {
          hash = sha3_256(options.body)
        }
        const token = await signer.signHash(hash)
        if (token) {
          headers.set('Authorization', token)
        }
      }
    }
    const resp = await fetch(url, { ...defaultOptions, ...options, headers })
    const body: T = await readResponseBody(resp)
    if (resp.ok) {
      return body as T
    } else {
      const requestId = resp.headers.get('X-Request-Id')
      throw createHTTPError(resp.status, body, requestId)
    }
  }

  request.defaultOptions = Object.freeze(defaultOptions)
  request.get = <T>(
    path: string,
    params?: URLSearchParamsInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.GET,
      signal
    })
  }
  request.post = <T>(
    path: string,
    body?: object,
    headers?: HeadersInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, undefined, {
      method: RequestMethod.POST,
      body: body ? (encode(body, rfc8949EncodeOptions) as BufferSource) : null,
      headers: { ...headers, 'Content-Type': CBOR_MIME_TYPE },
      signal
    })
  }
  request.put = <T>(
    path: string,
    params?: URLSearchParamsInit,
    body?: object,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.PUT,
      body: body ? (encode(body, rfc8949EncodeOptions) as BufferSource) : null,
      headers: { 'Content-Type': CBOR_MIME_TYPE },
      signal
    })
  }
  request.patch = <T>(
    path: string,
    params?: URLSearchParamsInit,
    body?: object,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.PATCH,
      body: body ? (encode(body, rfc8949EncodeOptions) as BufferSource) : null,
      headers: { 'Content-Type': CBOR_MIME_TYPE },
      signal
    })
  }
  request.delete = <T>(
    path: string,
    params?: URLSearchParamsInit,
    signal: AbortSignal | null | undefined = null
  ) => {
    return request<T>(path, params, {
      method: RequestMethod.DELETE,
      signal
    })
  }
  return request
}

type SupportedEncoding = 'gzip' | 'deflate' | 'deflate-raw'
const DecompressionStreamCtor = (globalThis as any).DecompressionStream as
  | undefined
  | (new (format: SupportedEncoding) => TransformStream)
async function readResponseBody<T>(resp: Response): Promise<T> {
  const encoding = normalizeEncoding(resp.headers.get('Content-Encoding'))
  const ct = resp.headers.get('Content-Type') || ''
  const isCBOR = ct.startsWith(CBOR_MIME_TYPE)
  const isJSON = ct.startsWith(JSON_MIME_TYPE)
  let body: unknown

  if (encoding && resp.body && typeof DecompressionStreamCtor === 'function') {
    const stream = resp.body.pipeThrough(new DecompressionStreamCtor(encoding))
    const decompressed = new Response(stream)
    body = isCBOR ? await decompressed.arrayBuffer() : await decompressed.text()
  } else {
    body = isCBOR ? await resp.arrayBuffer() : await resp.text()
  }

  if (ct.startsWith(CBOR_MIME_TYPE)) {
    return decode(new Uint8Array(body as ArrayBuffer)) as T
  } else if (isJSON) {
    const text: string = (body as string).trim()
    if (text) {
      try {
        return JSON.parse(text)
      } catch (err) {
        throw createHTTPError(
          resp.status,
          {
            message: 'Failed to parse JSON response',
            body
          },
          resp.headers.get('X-Request-Id')
        )
      }
    } else {
      body = null
    }
  }

  return body as T
}

function normalizeEncoding(value: string | null): SupportedEncoding | null {
  if (!value) return null
  const candidates = value
    .split(',')
    .map((part) => part.split(';')[0]?.trim().toLowerCase())
    .filter(Boolean)

  for (const candidate of candidates) {
    if (candidate === 'gzip' || candidate === 'x-gzip') return 'gzip'
    if (candidate === 'deflate' || candidate === 'x-deflate') return 'deflate'
    if (candidate === 'deflate-raw') return 'deflate-raw'
  }
  return null
}

//#region request error
function createHTTPError(
  status: number,
  body: unknown,
  requestId: string | null
): HTTPError {
  if (typeof body === 'object' && !!body) {
    if ('error' in body && typeof body.error === 'object') {
      return { ...body.error, requestId } as HTTPError
    }
    if ('message' in body && typeof body.message === 'string') {
      return {
        code: status,
        message: body.message,
        data: body,
        requestId
      } as HTTPError
    }
    return { code: status, message: JSON.stringify(body), requestId }
  }

  return { code: status, message: String(body), requestId }
}

export function mapToObj(m: any) {
  if (!(m instanceof Map)) {
    return m
  }

  const obj: any = {}
  for (const [key, val] of m) {
    obj[typeof key === 'string' ? key : String(key)] = val
  }
  return obj
}

export async function fetchFile(
  url: string,
  opts?: RequestInit
): Promise<Blob> {
  const resp = await fetch(url, opts)
  if (resp.status != 200) {
    throw new Error(await resp.text())
  }
  return await resp.blob()
}
