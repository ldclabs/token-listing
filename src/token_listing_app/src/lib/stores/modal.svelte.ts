import type { Component, Snippet } from 'svelte'

export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | 'full'
export type ModalCloseReason =
  | 'programmatic'
  | 'action'
  | 'backdrop'
  | 'escape'
  | 'close-button'

export interface ModalAction {
  id?: string
  label: string
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  disabled?: boolean
  autofocus?: boolean
  closeOnClick?: boolean
  handler?: (
    close: (reason?: ModalCloseReason) => void
  ) => void | false | Promise<void | false>
}

export interface ModalOptions {
  title?: string
  description?: string
  message?: string
  content?: Snippet
  component?: Component<any>
  componentProps?: Record<string, unknown>
  size?: ModalSize
  dismissable?: boolean
  closeOnBackdrop?: boolean
  closeOnEscape?: boolean
  actions?: ModalAction[]
  onClose?: (reason: ModalCloseReason) => void
  onOpen?: () => void
}

export interface ModalInstance extends ModalOptions {
  id: number
  close: (reason?: ModalCloseReason) => void
  actions: ModalAction[]
  dismissable: boolean
  closeOnBackdrop: boolean
  closeOnEscape: boolean
}

export const modalStore = $state<ModalInstance[]>([])

let idCounter = 0

// Usage example:
// ```js
// function onOpenModal() {
//   const { close } = showModal({
//     title: 'Confirm payment',
//     message: 'Are you sure you want to proceed?',
//     actions: [
//       { label: 'Cancel', variant: 'ghost' },
//       {
//         label: 'Confirm',
//         variant: 'primary',
//         handler: () => {
//           console.log('User confirmed payment')
//         }
//       }
//     ]
//   })
// }
// ```
export function showModal(options: ModalOptions): ModalInstance {
  const id = ++idCounter
  const instance: ModalInstance = {
    id,
    ...options,
    dismissable: options.dismissable ?? true,
    closeOnBackdrop: options.closeOnBackdrop ?? true,
    closeOnEscape: options.closeOnEscape ?? true,
    actions:
      options.actions?.map((action) => ({
        ...action,
        closeOnClick: action.closeOnClick ?? true
      })) ?? [],
    close: (reason = 'programmatic') => closeModal(id, reason)
  }

  modalStore.push(instance)
  queueMicrotask(() => options.onOpen?.())

  return instance
}

export function closeModal(
  id: number,
  reason: ModalCloseReason = 'programmatic'
) {
  const idx = modalStore.findIndex((modal) => modal.id === id)
  if (idx === -1) return

  const [modal] = modalStore.splice(idx, 1)
  modal?.onClose?.(reason)
}

export function closeTopModal(reason: ModalCloseReason = 'programmatic') {
  const modal = modalStore.at(-1)
  if (!modal) return
  closeModal(modal.id, reason)
}

export function clearModals(reason: ModalCloseReason = 'programmatic') {
  if (!modalStore.length) return
  const ids = modalStore.map((modal) => modal.id)
  ids.forEach((id) => closeModal(id, reason))
}

export function hasOpenModal() {
  return modalStore.length > 0
}
