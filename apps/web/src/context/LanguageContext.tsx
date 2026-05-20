import React, { createContext, useContext, useMemo, useState } from 'react'
import { LANGUAGE_OPTIONS, type Language, translations } from '../i18n/translations'

type LanguageContextValue = {
  language: Language
  setLanguage: (language: Language) => void
  t: (key: string, params?: Record<string, string | number>) => string
  formatCategory: (value: string) => string
  formatStatus: (value: string) => string
  formatPriority: (value: string) => string
  locale: string
  languages: Array<{ code: Language; label: string }>
}

const LanguageContext = createContext<LanguageContextValue | null>(null)
const STORAGE_KEY = 'civic_language'
const LOCALE_MAP: Record<Language, string> = {
  en: 'en-US',
  th: 'th-TH',
  ja: 'ja-JP',
}

function humanize(value: string): string {
  return value
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
}

function readStoredLanguage(): Language {
  const raw = localStorage.getItem(STORAGE_KEY)
  if (raw === 'th' || raw === 'en' || raw === 'ja') {
    return raw
  }
  if (typeof navigator !== 'undefined') {
    const browserLanguage = navigator.language.toLowerCase()
    if (browserLanguage.startsWith('th')) return 'th'
    if (browserLanguage.startsWith('ja')) return 'ja'
  }
  return 'en'
}

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguageState] = useState<Language>(() => readStoredLanguage())

  const setLanguage = (nextLanguage: Language) => {
    setLanguageState(nextLanguage)
    localStorage.setItem(STORAGE_KEY, nextLanguage)
  }

  const t = (key: string, params?: Record<string, string | number>): string => {
    const value = translations[language][key]
    const raw = value ?? translations.en[key] ?? key
    if (!params) return raw

    return Object.entries(params).reduce((message, [paramKey, paramValue]) => {
      return message.replace(new RegExp(`\\{${paramKey}\\}`, 'g'), String(paramValue))
    }, raw)
  }

  const tokenLabel = (prefix: 'category' | 'status' | 'priority', value: string): string => {
    const normalized = value.toLowerCase()
    const key = `${prefix}.${normalized}`
    const translated = t(key)
    if (translated === key) {
      return humanize(value)
    }
    return translated
  }

  const contextValue = useMemo<LanguageContextValue>(
    () => ({
      language,
      setLanguage,
      t,
      formatCategory: (value: string) => tokenLabel('category', value),
      formatStatus: (value: string) => tokenLabel('status', value),
      formatPriority: (value: string) => tokenLabel('priority', value),
      locale: LOCALE_MAP[language],
      languages: LANGUAGE_OPTIONS,
    }),
    [language],
  )

  return <LanguageContext.Provider value={contextValue}>{children}</LanguageContext.Provider>
}

export function useI18n(): LanguageContextValue {
  const context = useContext(LanguageContext)
  if (!context) {
    throw new Error('useI18n must be used inside LanguageProvider')
  }
  return context
}
