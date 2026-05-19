import { Activity, CheckCircle, Clock, AlertTriangle, TrendingUp } from 'lucide-react'

export default function DashboardPage() {
  return (
    <div className="max-w-7xl mx-auto px-4 py-12">
      <h1 className="text-3xl font-bold gradient-text mb-8">Impact Dashboard</h1>

      {/* Stats Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <StatCard icon={Activity} label="Total Issues" value="0" trend="+0%" />
        <StatCard icon={Clock} label="Open Issues" value="0" trend="0" />
        <StatCard icon={CheckCircle} label="Resolved" value="0" trend="+0%" />
        <StatCard icon={AlertTriangle} label="Critical" value="0" trend="0" />
      </div>

      {/* Charts Placeholder */}
      <div className="grid md:grid-cols-2 gap-6">
        <div className="glass-panel p-6">
          <h3 className="text-lg font-semibold mb-4">Issues by Category</h3>
          <div className="h-64 flex items-center justify-center text-slate-500">
            <p>Chart will render here with real data</p>
          </div>
        </div>
        <div className="glass-panel p-6">
          <h3 className="text-lg font-semibold mb-4">Resolution Timeline</h3>
          <div className="h-64 flex items-center justify-center text-slate-500">
            <p>Chart will render here with real data</p>
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="glass-panel p-6 mt-6">
        <h3 className="text-lg font-semibold mb-4">Recent Activity</h3>
        <div className="text-center py-12 text-slate-500">
          <p>No issues reported yet. Be the first to make an impact!</p>
        </div>
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
