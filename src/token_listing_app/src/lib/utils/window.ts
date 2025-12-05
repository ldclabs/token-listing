import debounce from 'debounce'

const STR_UNDEFINED = 'undefined'

// NOTE: Use the function to guarantee it's re-evaluated between jsdom and node runtime for tests.
export const isWindowDefined = typeof window != STR_UNDEFINED
export const isDocumentDefined = typeof document != STR_UNDEFINED
export const isNotificationSupported =
  isWindowDefined && 'Notification' in globalThis
export const hasRequestAnimationFrame = () =>
  isWindowDefined && typeof globalThis['requestAnimationFrame'] != STR_UNDEFINED
export const noop = () => {}

let online = true
export const isOnline = () => online

// For node and React Native, `add/removeEventListener` doesn't exist on window.
const [onWindowEvent, offWindowEvent] =
  isWindowDefined && window.addEventListener
    ? [
        window.addEventListener.bind(window),
        window.removeEventListener.bind(window)
      ]
    : [noop, noop]

export const isVisible = () => {
  const visibilityState = isDocumentDefined && document.visibilityState
  return visibilityState == null || visibilityState !== 'hidden'
}

export const isActive = () => isOnline() && isVisible()

export const initFocus = (callback: (ev: Event) => void) => {
  // TODO: callback will be triggered 2 times when the page is focused.
  if (isDocumentDefined) {
    document.addEventListener('visibilitychange', callback)
  } else {
    onWindowEvent('focus', callback)
  }
  return () => {
    if (isDocumentDefined) {
      document.removeEventListener('visibilitychange', callback)
    }
    offWindowEvent('focus', callback)
  }
}

export const initReconnect = (
  onlineCallback: () => void = noop,
  offlineCallback: () => void = noop
) => {
  const onOnline = () => {
    online = true
    onlineCallback()
  }
  const onOffline = () => {
    online = false
    offlineCallback()
  }
  onWindowEvent('online', onOnline)
  onWindowEvent('offline', onOffline)
  return () => {
    offWindowEvent('online', onOnline)
    offWindowEvent('offline', onOffline)
  }
}

export const popupCenter = ({
  width,
  height
}: {
  width: number
  height: number
}): string => {
  if (!isWindowDefined || !window.top) {
    return ''
  }

  const {
    top: { innerWidth, innerHeight }
  } = window

  const y = innerHeight / 2 + screenY - height / 2
  const x = innerWidth / 2 + screenX - width / 2

  return `width=${width},height=${height},top=${y},left=${x}`
}

export function clickOutside(
  node?: HTMLElement | null,
  callback: () => void = noop
) {
  if (!node) return noop

  const handler = (ev: PointerEvent) => {
    if (!node.contains(ev.target as Node)) {
      callback()
    }
  }

  onWindowEvent('pointerup', handler)

  return () => {
    offWindowEvent('pointerup', handler)
  }
}

export function scrollOnHooks(
  node: HTMLElement,
  {
    onTop,
    onBottom,
    onMoveUp,
    onMoveDown,
    inMoveUpViewport,
    inMoveDownViewport,
    inViewportHasId = true,
    inViewportHasClass = ''
  }: {
    onTop?: (() => void) | undefined
    onBottom?: (() => void) | undefined
    onMoveUp?: (() => void) | undefined
    onMoveDown?: (() => void) | undefined
    inMoveUpViewport?: ((els: HTMLElement[]) => void) | undefined
    inMoveDownViewport?: ((els: HTMLElement[]) => void) | undefined
    inViewportHasId?: boolean
    inViewportHasClass?: string
  }
) {
  if (!node) return noop

  const callTop = onTop && debounce(onTop, 200, { immediate: false })
  const callBottom = onBottom && debounce(onBottom, 200, { immediate: false })
  const callMoveUp = onMoveUp && debounce(onMoveUp, 200, { immediate: false })
  const callMoveDown =
    onMoveDown && debounce(onMoveDown, 200, { immediate: false })
  const callInMoveUpViewport =
    inMoveUpViewport && debounce(inMoveUpViewport, 200, { immediate: false })
  const callInMoveDownViewport =
    inMoveDownViewport &&
    debounce(inMoveDownViewport, 200, { immediate: false })

  let lastScrollTop = 0
  const handler = (ev: Event) => {
    const target = ev.currentTarget as HTMLElement
    if (target.scrollTop > lastScrollTop) {
      callMoveUp && callMoveUp()
      if (callInMoveUpViewport) {
        let children = Array.from(target.children) as HTMLElement[]
        if (inViewportHasId || inViewportHasClass) {
          children = children.filter((el) => {
            if (inViewportHasId) {
              return !!el.id
            }
            return el.classList.contains(inViewportHasClass)
          })
        }
        const els = elementsInViewport(target, children)
        if (els.length > 0) {
          callInMoveUpViewport(els)
        }
      }
    } else {
      callMoveDown && callMoveDown()
      if (callInMoveDownViewport) {
        let children = Array.from(target.children) as HTMLElement[]
        if (inViewportHasId || inViewportHasClass) {
          children = children.filter((el) => {
            if (inViewportHasId) {
              return !!el.id
            }
            return el.classList.contains(inViewportHasClass)
          })
        }
        const els = elementsInViewport(target, children)
        if (els.length > 0) {
          callInMoveDownViewport(els)
        }
      }
    }

    if (target.scrollTop < lastScrollTop && target.scrollTop <= 5) {
      callTop && callTop()
    } else if (
      target.scrollTop > lastScrollTop &&
      target.clientHeight + target.scrollTop + 5 >= target.scrollHeight
    ) {
      callBottom && callBottom()
    }

    lastScrollTop = target.scrollTop
  }

  node.addEventListener('scroll', handler)
  return () => {
    node.removeEventListener('scroll', handler)
    callBottom && callBottom.clear()
    callMoveUp && callMoveUp.clear()
    callMoveDown && callMoveDown.clear()
    callInMoveUpViewport && callInMoveUpViewport.clear()
    callInMoveDownViewport && callInMoveDownViewport.clear()
  }
}

export function elementsInViewport(
  container: HTMLElement,
  els: HTMLElement[]
): HTMLElement[] {
  const containerRect = container.getBoundingClientRect()
  const rt: HTMLElement[] = []
  for (const el of els) {
    const rect = el.getBoundingClientRect()
    const threadhold = 0.5 * rect.height
    if (
      (rect.top >= containerRect.top &&
        rect.top + threadhold < containerRect.bottom) ||
      (rect.bottom <= containerRect.bottom &&
        rect.bottom - threadhold > containerRect.top)
    ) {
      rt.push(el)
    }
  }

  return rt
}

export function scrollIntoView(
  messageId: string,
  behavior: ScrollBehavior = 'instant',
  block: ScrollLogicalPosition = 'center'
): void {
  const ele = document.getElementById(messageId)

  if (ele) {
    ele.scrollIntoView({
      block,
      behavior
    })
  }
}

export function copyTextToClipboard(text: string): Promise<boolean> {
  if (!isWindowDefined) return Promise.resolve(false)

  if (navigator.clipboard && globalThis.isSecureContext) {
    return navigator.clipboard
      .writeText(text)
      .then(() => true)
      .catch(() => false)
  }

  try {
    const ta = document.createElement('textarea')
    ta.value = text
    ta.setAttribute('readonly', '')
    ta.style.position = 'fixed'
    ta.style.top = '-9999px'
    document.body.appendChild(ta)
    ta.select()
    const ok = document.execCommand('copy')
    document.body.removeChild(ta)
    return Promise.resolve(ok)
  } catch (e) {
    return Promise.resolve(false)
  }
}

export function isInMobileBrowser(): boolean {
  if (!isWindowDefined || !navigator || !navigator.userAgent) return false

  const ua = navigator.userAgent || ''

  // Common mobile indicators, excluding some tablet/desktop cases where possible.
  const isAndroid = /Android/i.test(ua)
  const isIOS = /(iPhone|iPod)/i.test(ua)
  const isIPad = /iPad/i.test(ua)
  const isMobileKeyword = /Mobile/i.test(ua)

  // iPad on iOS 13+ reports as MacIntel with touch; we treat that as mobile browser.
  const isTouchMac =
    !isAndroid &&
    !isIOS &&
    !isIPad &&
    typeof navigator.maxTouchPoints === 'number' &&
    navigator.maxTouchPoints > 1 &&
    /Macintosh/i.test(ua)

  if (isIOS || isAndroid) return true
  if (isIPad && isMobileKeyword) return true
  if (isTouchMac) return true

  return false
}

export function navLang(): string {
  if (!isWindowDefined) return 'en'
  return lang639(navigator.language || 'en')
}

export function navLangs(): string[] {
  if (!isWindowDefined) return ['en']
  const langs = navigator.languages.map(lang639)
  // ['zh', 'en', 'ar', 'zh'] -> ['zh', 'en', 'ar']
  return [...new Set(langs)]
}

function lang639(lang: string): string {
  const idx = lang.indexOf('-')
  if (idx < 0) return lang.toLowerCase()
  return lang.slice(0, idx).toLowerCase()
}
