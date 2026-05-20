import { Link } from 'react-router-dom'
import { Shield, Zap, Users, BarChart3, ArrowRight, Activity } from 'lucide-react'
import { useAnalytics } from '../hooks/useAnalytics'
import { useI18n } from '../context/LanguageContext'

export default function HomePage() {
  const { data: analytics, isLoading: analyticsLoading } = useAnalytics()
  const { t } = useI18n()

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      {/* Hero Section */}
      <div className="text-center py-16 lg:py-24">
        <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full bg-sky-500/10 border border-sky-500/20 mb-8">
          <Activity className="w-4 h-4 text-sky-300" />
          <span className="text-sm text-sky-300 font-medium">{t('home.badge')}</span>
        </div>

        <h1 className="text-4xl md:text-6xl font-bold mb-6">
          <span className="gradient-text">{t('home.title.line1')}</span>
          <br />
          <span className="text-white">{t('home.title.line2')}</span>
        </h1>

        <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10">
          {t('home.subtitle')}
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link to="/report" className="btn-primary flex items-center space-x-2">
            <span>{t('home.cta.report')}</span>
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link to="/dashboard" className="btn-secondary">
            {t('home.cta.dashboard')}
          </Link>
        </div>
      </div>

      {/* Stats Section */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-16">
        {analyticsLoading ? (
          <>
            <StatSkeleton />
            <StatSkeleton />
            <StatSkeleton />
            <StatSkeleton />
          </>
        ) : (
          <>
            <HomeStat label={t('home.stats.total')} value={analytics?.total_issues ?? 0} />
            <HomeStat label={t('home.stats.open')} value={analytics?.open_issues ?? 0} />
            <HomeStat label={t('home.stats.resolved')} value={analytics?.resolved_issues ?? 0} />
            <HomeStat
              label={t('home.stats.impact')}
              value={analytics?.impact_score?.toFixed(1) ?? '0.0'}
            />
          </>
        )}
      </div>

      {/* Features Grid */}
      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 py-16">
        <FeatureCard
          icon={Shield}
          title={t('home.feature.simple.title')}
          description={t('home.feature.simple.desc')}
        />
        <FeatureCard
          icon={Zap}
          title={t('home.feature.response.title')}
          description={t('home.feature.response.desc')}
        />
        <FeatureCard
          icon={Users}
          title={t('home.feature.community.title')}
          description={t('home.feature.community.desc')}
        />
        <FeatureCard
          icon={BarChart3}
          title={t('home.feature.transparent.title')}
          description={t('home.feature.transparent.desc')}
        />
      </div>

      {/* Open Source Section */}
      <div className="glass-panel p-8 rounded-2xl my-16">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">{t('home.open.title')}</h2>
          <p className="text-slate-400 max-w-xl mx-auto mb-6">
            {t('home.open.desc')}
          </p>
          <div className="flex flex-wrap justify-center gap-4">
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">{t('home.open.badge.license')}</span>
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">{t('home.open.badge.selfHost')}</span>
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">{t('home.open.badge.community')}</span>
          </div>
        </div>
      </div>
    </div>
  )
}

function HomeStat({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="glass-panel p-4 text-center">
      <div className="text-2xl md:text-3xl font-bold gradient-text">{value}</div>
      <div className="text-sm text-slate-400 mt-1">{label}</div>
    </div>
  )
}

function StatSkeleton() {
  return (
    <div className="glass-panel p-4 text-center animate-pulse">
      <div className="w-16 h-8 bg-slate-700 rounded mx-auto mb-2" />
      <div className="w-20 h-4 bg-slate-700 rounded mx-auto" />
    </div>
  )
}

function FeatureCard({ icon: Icon, title, description }: { icon: React.ElementType; title: string; description: string }) {
  return (
    <div className="glass-panel p-6 hover:border-sky-500/30 transition-all duration-300">
      <div className="w-12 h-12 rounded-lg bg-sky-500/10 flex items-center justify-center mb-4">
        <Icon className="w-6 h-6 text-sky-300" />
      </div>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <p className="text-slate-400 text-sm">{description}</p>
    </div>
  )
}
