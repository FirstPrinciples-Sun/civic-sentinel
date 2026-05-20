import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || ''

export const api = axios.create({
  baseURL: `${API_BASE_URL}/api/v1`,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Attach JWT token to every request
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('civic_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

let isRefreshing = false
let failedQueue: any[] = []

const processQueue = (error: any, token: string | null = null) => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error)
    } else {
      prom.resolve(token)
    }
  })
  failedQueue = []
}

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    if (error.response?.status === 401 && !originalRequest._retry) {
      if (isRefreshing) {
        return new Promise((resolve, reject) => {
          failedQueue.push({ resolve, reject })
        })
          .then((token) => {
            originalRequest.headers.Authorization = `Bearer ${token}`
            return api(originalRequest)
          })
          .catch((err) => Promise.reject(err))
      }

      originalRequest._retry = true
      isRefreshing = true

      const refreshToken = localStorage.getItem('civic_refresh')
      if (refreshToken) {
        try {
          const response = await axios.post(`${API_BASE_URL}/api/v1/auth/refresh`, {
            refresh_token: refreshToken,
          })

          const { access_token, refresh_token, user } = response.data
          localStorage.setItem('civic_token', access_token)
          localStorage.setItem('civic_refresh', refresh_token)
          if (user) {
            localStorage.setItem('civic_user', JSON.stringify(user))
          }

          api.defaults.headers.common['Authorization'] = `Bearer ${access_token}`
          originalRequest.headers.Authorization = `Bearer ${access_token}`

          processQueue(null, access_token)
          return api(originalRequest)
        } catch (refreshError) {
          processQueue(refreshError, null)
          
          localStorage.removeItem('civic_token')
          localStorage.removeItem('civic_refresh')
          localStorage.removeItem('civic_user')
          
          window.location.href = '/login'
          return Promise.reject(refreshError)
        } finally {
          isRefreshing = false
        }
      } else {
        localStorage.removeItem('civic_token')
        localStorage.removeItem('civic_refresh')
        localStorage.removeItem('civic_user')
        window.location.href = '/login'
      }
    }

    return Promise.reject(error)
  }
)

export interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
  error?: string
}

export interface GeoLocation {
  latitude: number
  longitude: number
  address?: string | null
}

export interface Issue {
  id: string
  title: string
  description: string
  category: string
  priority: string
  status: string
  location: GeoLocation
  reporter_id: string | null
  assigned_to: string | null
  media_urls: string[]
  tags: string[]
  created_at: string
  updated_at: string
  resolved_at: string | null
}

export interface CategoryCount {
  category: string
  count: number
}

export interface PriorityCount {
  priority: string
  count: number
}

export interface AnalyticsData {
  total_issues: number
  open_issues: number
  resolved_issues: number
  avg_resolution_hours: number
  issues_by_category: CategoryCount[]
  issues_by_priority: PriorityCount[]
  impact_score: number
}

export interface IssuesListMeta {
  total: number
  page: number
  per_page: number
}

export interface IssuesListResponse {
  success: boolean
  data: Issue[]
  meta: IssuesListMeta
}

export interface CreateIssueRequest {
  title: string
  description: string
  category: string
  location: GeoLocation
  media_urls?: string[]
  tags?: string[]
}
