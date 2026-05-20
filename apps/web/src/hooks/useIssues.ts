import { useQuery } from '@tanstack/react-query'
import {
  api,
  type ApiResponse,
  type IssueFilters,
  type Issue,
  type IssueComment,
  type IssuesListResponse,
  type IssueStatusHistoryEntry,
  type UpdateIssueRequest,
} from '../services/api'

export interface UseIssuesResult {
  issues: Issue[]
  meta: {
    total: number
    page: number
    per_page: number
    total_pages: number
  } | null
}

export function useIssues(page = 1, perPage = 20, filters: IssueFilters = {}) {
  return useQuery<UseIssuesResult, Error>({
    queryKey: ['issues', page, perPage, filters],
    queryFn: async () => {
      const response = await api.get<IssuesListResponse>('/issues', {
        params: { page, limit: perPage, ...filters },
      })
      return {
        issues: response.data.data,
        meta: response.data.meta ?? null,
      }
    },
  })
}

export function useIssue(id: string | undefined) {
  return useQuery<Issue, Error>({
    queryKey: ['issue', id],
    queryFn: async () => {
      const response = await api.get<{ success: boolean; data: Issue }>(`/issues/${id}`)
      return response.data.data
    },
    enabled: !!id,
  })
}

export function useIssueComments(id: string | undefined) {
  return useQuery<IssueComment[], Error>({
    queryKey: ['issue-comments', id],
    queryFn: async () => {
      const response = await api.get<ApiResponse<IssueComment[]>>(`/issues/${id}/comments`)
      return response.data.data
    },
    enabled: !!id,
  })
}

export function useIssueHistory(id: string | undefined) {
  return useQuery<IssueStatusHistoryEntry[], Error>({
    queryKey: ['issue-history', id],
    queryFn: async () => {
      const response = await api.get<ApiResponse<IssueStatusHistoryEntry[]>>(`/issues/${id}/history`)
      return response.data.data
    },
    enabled: !!id,
  })
}

export async function updateIssueStatus(issueId: string, payload: UpdateIssueRequest) {
  const response = await api.patch<ApiResponse<Issue>>(`/issues/${issueId}`, payload)
  return response.data.data
}
