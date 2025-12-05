export function portal(
  node: HTMLElement,
  target: HTMLElement | string = 'body'
) {
  const targetEl =
    typeof target === 'string'
      ? (document.querySelector(target) as HTMLElement | null)
      : target
  if (!targetEl) return {}

  targetEl.appendChild(node)

  return {
    destroy() {
      if (node.parentNode === targetEl) targetEl.removeChild(node)
    }
  }
}
