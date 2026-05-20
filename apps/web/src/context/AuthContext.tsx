import React, { createContext, useContext, useState, useCallback } from 'react'
import { api } from '../services/api'

export interface User {
  id: string
  email: string
  name: string | null
  role: string
}

interface AuthResponse {
  success: boolean
  access_token: string
  refresh_token: string
  token_type: string
  expires_in: number
  user: User
}

interface AuthContextType {
  user: User | null
  token: string | null
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string, name: string) => Promise<void>
  logout: () => Promise<void>
  isLoading: boolean
  isAuthenticated: boolean
}

const AuthContext = createContext<AuthContextType | null>(null)

function readStoredUser(): User | null {
  const raw = localStorage.getItem('civic_user')
  if (!raw) return null

  try {
    const parsed = JSON.parse(raw) as User
    if (!parsed || typeof parsed !== 'object' || typeof parsed.id !== 'string') {
      localStorage.removeItem('civic_user')
      return null
    }
    return parsed
  } catch {
    localStorage.removeItem('civic_user')
    return null
  }
}

function readStoredToken(): string | null {
  const token = localStorage.getItem('civic_token')
  if (!token || token === 'undefined' || token === 'null') {
    localStorage.removeItem('civic_token')
    return null
  }
  return token
}

export function useAuth(): AuthContextType {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be inside AuthProvider')
  return ctx
}

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(() => readStoredUser())
  const [token, setToken] = useState<string | null>(() => readStoredToken())
  const [isLoading, setIsLoading] = useState(false)
  const isAuthenticated = !!token

  const login = useCallback(async (email: string, password: string) => {
    setIsLoading(true)
    try {
      const { data } = await api.post<AuthResponse>('/auth/login', { email, password })
      localStorage.setItem('civic_token', data.access_token)
      localStorage.setItem('civic_refresh', data.refresh_token)
      localStorage.setItem('civic_user', JSON.stringify(data.user))
      setToken(data.access_token)
      setUser(data.user)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const register = useCallback(async (email: string, password: string, name: string) => {
    setIsLoading(true)
    try {
      const { data } = await api.post<AuthResponse>('/auth/register', { email, password, name })
      localStorage.setItem('civic_token', data.access_token)
      localStorage.setItem('civic_refresh', data.refresh_token)
      localStorage.setItem('civic_user', JSON.stringify(data.user))
      setToken(data.access_token)
      setUser(data.user)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const logout = useCallback(async () => {
    const refresh = localStorage.getItem('civic_refresh')
    if (refresh) {
      try {
        await api.post('/auth/logout', { refresh_token: refresh })
      } catch { /* ignore */ }
    }
    localStorage.removeItem('civic_token')
    localStorage.removeItem('civic_refresh')
    localStorage.removeItem('civic_user')
    setToken(null)
    setUser(null)
  }, [])

  return (
    <AuthContext.Provider value={{ user, token, login, register, logout, isLoading, isAuthenticated }}>
      {children}
    </AuthContext.Provider>
  )
}
