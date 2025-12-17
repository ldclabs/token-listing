const src = globalThis.location?.href || ''

export const APP_VERSION = '0.1.0'
export const IS_LOCAL = src.includes('localhost') || src.includes('127.0.0.1')
export const ENV = IS_LOCAL ? 'local' : 'ic'

export const INTERNET_IDENTITY_CANISTER_ID = 'rdmx6-jaaaa-aaaaa-aaadq-cai' // ic & local
export const TOKEN_LISTING = 'km2n7-vaaaa-aaaap-an57a-cai'
export const PAYING_ENDPOINT = 'https://api.1pay.ing'
