import { useQuery } from '@tanstack/react-query'
import { api, type ApiResponse, type AnalyticsData } from '../services/api'

export function useAnalytics() {
  return useQuery<AnalyticsData, Error>({
    queryKey: ['analytics'],
    queryFn: async () => {
      const response = await api.get<ApiResponse<AnalyticsData>>('/analytics')
      return response.data.data
    },
  })
}
