import { Activity, CheckCircle, Clock, AlertTriangle, TrendingUp, Loader2 } from 'lucide-react'
import { useAnalytics } from '../hooks/useAnalytics'
import { useIssues } from '../hooks/useIssues'
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from 'recharts'

const CATEGORY_COLORS: Record<string, string> = {
  infrastructure: '#10b981',
  safety: '#ef4444',
  environment: '#3b82f6',
  sanitation: '#f59e0b',
  transportation: '#8b5cf6',
  public_utility: '#06b6d4',
  other: '#64748b',
}

function formatCategory(category: string): string {
  return category.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase())
}

function formatDate(dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function getStatusColor(status: string): string {
  switch (status) {
    case 'resolved':
      return 'text-emerald-400 bg-emerald-500/10'
    case 'inprogress':
      return 'text-blue-400 bg-blue-500/10'
    case 'escalated':
      return 'text-red-400 bg-red-500/10'
    case 'underreview':
      return 'text-amber-400 bg-amber-500/10'
    default:
      return 'text-slate-400 bg-slate-500/10'
  }
}

export default function DashboardPage() {
  const { data: analytics, isLoading: analyticsLoading, error: analyticsError } = useAnalytics()
  const { data: issues, isLoading: issuesLoading, error: issuesError } = useIssues(1, 10)

  const categoryData = analytics?.issues_by_category.map((item) => ({
    name: formatCategory(item.category),
    count: item.count,
    rawCategory: item.category,
  })) ?? []

  const criticalCount =
    analytics?.issues_by_priority.find((p) => p.priority === 'critical')?.count ?? 0

  return (
    <div className="max-w-7xl mx-auto px-4 py-12">
      <h1 className="text-3xl font-bold gradient-text mb-8">Impact Dashboard</h1>

      {/* Stats Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        {analyticsLoading ? (
          <>
            <StatCardSkeleton />
            <StatCardSkeleton />
            <StatCardSkeleton />
            <StatCardSkeleton />
          </>
        ) : analyticsError ? (
          <div className="col-span-full glass-panel p-6 text-center text-red-400">
            <AlertTriangle className="w-6 h-6 mx-auto mb-2" />
            <p>Failed to load analytics data.</p>
          </div>
        ) : (
          <>
            <StatCard
              icon={Activity}
              label="Total Issues"
              value={String(analytics?.total_issues ?? 0)}
              trend="+0%"
            />
            <StatCard
              icon={Clock}
              label="Open Issues"
              value={String(analytics?.open_issues ?? 0)}
              trend="0"
            />
            <StatCard
              icon={CheckCircle}
              label="Resolved"
              value={String(analytics?.resolved_issues ?? 0)}
              trend="+0%"
            />
            <StatCard
              icon={AlertTriangle}
              label="Critical"
              value={String(criticalCount)}
              trend="0"
            />
          </>
        )}
      </div>

      {/* Charts */}
      <div className="grid md:grid-cols-2 gap-6">
        <div className="glass-panel p-6">
          <h3 className="text-lg font-semibold mb-4">Issues by Category</h3>
          {analyticsLoading ? (
            <div className="h-64 flex items-center justify-center">
              <Loader2 className="w-8 h-8 animate-spin text-emerald-400" />
            </div>
          ) : analyticsError ? (
            <div className="h-64 flex items-center justify-center text-red-400 text-sm">
              Failed to load chart data
            </div>
          ) : categoryData.length === 0 ? (
            <div className="h-64 flex items-center justify-center text-slate-500">
              <p>No data available</p>
            </div>
          ) : (
            <ResponsiveContainer width="100%" height={256}>
              <BarChart data={categoryData}>
                <XAxis dataKey="name" tick={{ fill: '#94a3b8', fontSize: 12 }} axisLine={false} tickLine={false} />
                <YAxis tick={{ fill: '#94a3b8', fontSize: 12 }} axisLine={false} tickLine={false} allowDecimals={false} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: '#1e293b',
                    border: '1px solid #334155',
                    borderRadius: '8px',
                    color: '#f8fafc',
                  }}
                  cursor={{ fill: '#334155', opacity: 0.3 }}
                />
                <Bar dataKey="count" radius={[4, 4, 0, 0]}>
                  {categoryData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={CATEGORY_COLORS[entry.rawCategory] ?? '#64748b'} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          )}
        </div>

        <div className="glass-panel p-6">
          <h3 className="text-lg font-semibold mb-4">Resolution Timeline</h3>
          <div className="h-64 flex items-center justify-center text-slate-500">
            <div className="text-center">
              <TrendingUp className="w-10 h-10 mx-auto mb-3 text-slate-600" />
              <p className="text-sm">Coming soon with historical data</p>
              <p className="text-xs text-slate-600 mt-1">
                Avg resolution: {analytics?.avg_resolution_hours?.toFixed(1) ?? 0}h
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="glass-panel p-6 mt-6">
        <h3 className="text-lg font-semibold mb-4">Recent Activity</h3>
        {issuesLoading ? (
          <div className="flex justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-emerald-400" />
          </div>
        ) : issuesError ? (
          <div className="text-center py-12 text-red-400">
            <AlertTriangle className="w-6 h-6 mx-auto mb-2" />
            <p>Failed to load recent activity.</p>
          </div>
        ) : !issues || issues.length === 0 ? (
          <div className="text-center py-12 text-slate-500">
            <p>No issues reported yet. Be the first to make an impact!</p>
          </div>
        ) : (
          <div className="space-y-3">
            {issues.map((issue) => (
              <div
                key={issue.id}
                className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700/50 hover:border-slate-600/50 transition-all"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span
                      className={`text-xs px-2 py-0.5 rounded-full capitalize ${getStatusColor(issue.status)}`}
                    >
                      {issue.status.replace(/_/g, ' ')}
                    </span>
                    <span className="text-xs text-slate-500 capitalize">
                      {formatCategory(issue.category)}
                    </span>
                  </div>
                  <p className="text-sm font-medium text-slate-200 truncate">{issue.title}</p>
                  <p className="text-xs text-slate-500">{formatDate(issue.created_at)}</p>
                </div>
                <span
                  className={`text-xs font-medium px-2 py-1 rounded ml-4 ${
                    issue.priority === 'critical'
                      ? 'text-red-400 bg-red-500/10'
                      : issue.priority === 'high'
                        ? 'text-amber-400 bg-amber-500/10'
                        : 'text-slate-400 bg-slate-500/10'
                  }`}
                >
                  {issue.priority}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function StatCard({
  icon: Icon,
  label,
  value,
  trend,
}: {
  icon: React.ElementType
  label: string
  value: string
  trend: string
}) {
  return (
    <div className="glass-panel p-4">
      <div className="flex items-center justify-between mb-2">
        <Icon className="w-5 h-5 text-emerald-400" />
        <span className="text-xs text-emerald-400">{trend}</span>
      </div>
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-sm text-slate-400">{label}</div>
    </div>
  )
}

function StatCardSkeleton() {
  return (
    <div className="glass-panel p-4 animate-pulse">
      <div className="flex items-center justify-between mb-2">
        <div className="w-5 h-5 bg-slate-700 rounded" />
        <div className="w-8 h-3 bg-slate-700 rounded" />
      </div>
      <div className="w-12 h-7 bg-slate-700 rounded mb-1" />
      <div className="w-20 h-4 bg-slate-700 rounded" />
    </div>
  )
}
