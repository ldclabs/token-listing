type Theme = 'light' | 'dark'
let themeValue = $state<Theme>('light')

const saveTheme = (value: Theme) => {
  themeValue = value
  localStorage['theme'] = value

  document.documentElement.classList.toggle(
    'dark',
    localStorage['theme'] === 'dark' ||
      (!('theme' in localStorage) &&
        window.matchMedia('(prefers-color-scheme: dark)').matches)
  )
}
export const getTheme = () => themeValue

export const toggleTheme = () =>
  saveTheme(themeValue === 'dark' ? 'light' : 'dark')

export function initTheme() {
  saveTheme(localStorage['theme'] || 'light')
}
