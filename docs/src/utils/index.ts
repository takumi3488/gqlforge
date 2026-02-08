export const analyticsHandler = (_category: string, _action: string, _label: string): void => {}

export const setBodyOverflow = (value: "initial" | "hidden") => {
  document.body.style.overflow = value
}

export const isValidURL = (url: string) => {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}
