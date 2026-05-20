import { useQuery } from '@tanstack/react-query'
import { api, type Issue, type IssuesListResponse } from '../services/api'

export function useIssues(page = 1, perPage = 20) {
  return useQuery<Issue[], Error>({
    queryKey: ['issues', page, perPage],
    queryFn: async () => {
      const response = await api.get<IssuesListResponse>('/issues', {
        params: { page, limit: perPage },
      })
      return response.data.data
    },
  })
}
