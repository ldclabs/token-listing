const locale = new Intl.Locale(globalThis?.navigator.language || 'en')

export interface Token {
  symbol: string
  name: string
  decimals: number
  logo?: string
}

export class TokenAmount {
  private constructor(
    protected ulps: bigint,
    public token: Token
  ) {}

  public static fromUlps({
    amount,
    token
  }: {
    amount: bigint
    token: Token
  }): TokenAmount {
    return new TokenAmount(amount, token)
  }

  public static fromString({
    amount,
    token
  }: {
    amount: string
    token: Token
  }): TokenAmount | FromStringToTokenError {
    const ulps = convertStringToUlps({ amount, decimals: token.decimals })

    if (typeof ulps === 'bigint') {
      return new TokenAmount(ulps, token)
    }
    return ulps
  }

  public static fromNumber({
    amount,
    token
  }: {
    amount: number
    token: Token
  }): TokenAmount {
    const tokenAmount = TokenAmount.fromString({
      amount: amount.toFixed(token.decimals),
      token
    })
    if (tokenAmount instanceof TokenAmount) {
      return tokenAmount
    }
    if (tokenAmount === FromStringToTokenError.FractionalTooManyDecimals) {
      throw new Error(
        `Number ${amount} has more than ${token.decimals} decimals`
      )
    }

    // This should never happen
    throw new Error(`Invalid number ${amount}`)
  }

  public toUlps(): bigint {
    return this.ulps
  }

  public toE8s(): bigint {
    if (this.token.decimals < 8) {
      return this.ulps * 10n ** BigInt(8 - this.token.decimals)
    }
    if (this.token.decimals === 8) {
      return this.ulps
    }
    return this.ulps / 10n ** BigInt(this.token.decimals - 8)
  }
}

export interface TokenInfo extends Token {
  fee: bigint
  one: bigint
  logo: string
  address: string
  maxDigits?: number
}

export const PANDAToken: TokenInfo = {
  name: 'ICPanda',
  symbol: 'PANDA',
  decimals: 8,
  fee: 10000n,
  one: 100000000n,
  logo: 'https://tokenlist.ing/_assets/images/panda.webp',
  address: 'druyg-tyaaa-aaaaq-aactq-cai',
  maxDigits: 4
}

export const USDCToken: TokenInfo = {
  name: 'Circle',
  symbol: 'USDC',
  decimals: 6,
  fee: 0n,
  one: 1000000n,
  logo: 'https://tokenlist.ing/_assets/images/usdc.webp',
  address: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
  maxDigits: 4
}

export function formatNumber(val: number, maxDigits: number = 4): string {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDigits,
    roundingMode: 'trunc'
  } as Intl.NumberFormatOptions).format(val)
}

export function formatAmount(
  amount: bigint,
  decimals: number,
  maxDigits: number = 6
): string {
  const decimalVal = 10n ** BigInt(decimals)
  const integerPart = amount / decimalVal
  const fractionalPart = amount % decimalVal
  const val = Number(integerPart) + Number(fractionalPart) / Number(decimalVal)
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: maxDigits,
    roundingMode: 'trunc'
  } as Intl.NumberFormatOptions).format(val)
}

export class TokenDisplay {
  readonly billedToSource: boolean
  readonly token: TokenInfo
  readonly decimals: bigint
  readonly one: bigint
  readonly formater: Intl.NumberFormat

  amount: bigint
  fee: bigint

  // Initialize from a string. Accepted formats:
  //   1234567.8901
  //   1'234'567.8901
  //   1,234,567.8901
  //
  static fromString(
    token: TokenInfo,
    amount: string,
    billedToSource: boolean = true
  ): TokenDisplay {
    const val = TokenAmount.fromString({ amount, token }) as TokenAmount
    return new TokenDisplay(token, val.toUlps(), billedToSource)
  }

  // Initialize from a number.
  // 1 integer is considered 10^{token.decimals} ulps
  static fromNumber(
    token: TokenInfo,
    amount: number,
    billedToSource: boolean = true
  ): TokenDisplay {
    const val = TokenAmount.fromNumber({ amount, token }) as TokenAmount
    return new TokenDisplay(token, val.toUlps(), billedToSource)
  }

  constructor(
    token: TokenInfo,
    amount: bigint,
    billedToSource: boolean = true
  ) {
    this.billedToSource = billedToSource
    this.token = token
    this.decimals = BigInt(token.decimals)
    this.one = 10n ** this.decimals
    this.formater = new Intl.NumberFormat(locale, {
      minimumFractionDigits: 1,
      maximumFractionDigits: 4,
      roundingMode: 'floor'
    } as Intl.NumberFormatOptions)
    this.amount = amount
    this.fee = token.fee
  }

  get num(): number {
    const integerPart = this.amount / this.one
    const fractionalPart = this.amount % this.one
    const val = Number(integerPart) + Number(fractionalPart) / Number(this.one)
    return val
  }

  set num(amount: number) {
    const val = TokenAmount.fromNumber({
      amount,
      token: this.token
    }) as TokenAmount
    this.amount = val.toUlps()
  }

  get total(): bigint {
    return this.billedToSource ? this.amount + this.fee : this.amount
  }

  get received(): bigint {
    return this.billedToSource ? this.amount : this.amount - this.fee
  }

  withAmount(amount: bigint): TokenDisplay {
    return new TokenDisplay(this.token, amount, this.billedToSource)
  }

  parseAmount(amount: string | number): bigint {
    const val =
      typeof amount === 'string'
        ? (TokenAmount.fromString({ amount, token: this.token }) as TokenAmount)
        : (TokenAmount.fromNumber({
            amount,
            token: this.token
          }) as TokenAmount)
    return val.toUlps()
  }

  fullFormat(value: number | bigint): string {
    return this.formater.format(value)
  }

  short(maxDigits: number = 3): string {
    return formatNumber(this.num, maxDigits)
  }

  toString(): string {
    return this.fullFormat(this.num)
  }

  display(): string {
    return this.toString()
  }

  displayValue(value: bigint): string {
    const integerPart = value / this.one
    const fractionalPart = value % this.one
    const val = Number(integerPart) + Number(fractionalPart) / Number(this.one)
    return this.fullFormat(val)
  }

  displayFee(): string {
    return this.displayValue(this.fee)
  }

  displayTotal(): string {
    return this.displayValue(this.total)
  }

  displayReceived(): string {
    return this.displayValue(this.received)
  }
}

export enum FromStringToTokenError {
  FractionalMoreThan8Decimals,
  InvalidFormat,
  FractionalTooManyDecimals
}

function convertStringToUlps({
  amount,
  decimals
}: {
  amount: string
  decimals: number
}): bigint | FromStringToTokenError {
  // Remove all instances of "," and "'".
  amount = amount.trim().replace(/[,']/g, '')

  // Verify that the string is of the format 1234.5678
  const regexMatch = amount.match(/\d*(\.\d*)?/)
  if (!regexMatch || regexMatch[0] !== amount) {
    return FromStringToTokenError.InvalidFormat
  }

  const [integral, fractional] = amount.split('.')

  let ulps = 0n
  const ulpsPerToken = 10n ** BigInt(decimals)

  if (integral) {
    try {
      ulps += BigInt(integral) * ulpsPerToken
    } catch {
      return FromStringToTokenError.InvalidFormat
    }
  }

  if (fractional) {
    if (fractional.length > decimals) {
      return FromStringToTokenError.FractionalTooManyDecimals
    }
    try {
      ulps += BigInt(fractional.padEnd(decimals, '0'))
    } catch {
      return FromStringToTokenError.InvalidFormat
    }
  }

  return ulps
}
