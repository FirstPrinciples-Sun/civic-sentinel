import axios from 'axios'

const API_BASE_URL = import.meta.env.VITE_API_URL || ''

export const api = axios.create({
  baseURL: `${API_BASE_URL}/api/v1`,
  headers: {
    'Content-Type': 'application/json',
  },
})

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
