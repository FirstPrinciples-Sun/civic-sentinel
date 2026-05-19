import { Link } from 'react-router-dom'
import { Shield, Zap, Users, BarChart3, ArrowRight, Activity } from 'lucide-react'

export default function HomePage() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      {/* Hero Section */}
      <div className="text-center py-16 lg:py-24">
        <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20 mb-8">
          <Activity className="w-4 h-4 text-emerald-400" />
          <span className="text-sm text-emerald-400 font-medium">Open Source</span>
        </div>
        
        <h1 className="text-4xl md:text-6xl font-bold mb-6">
          <span className="gradient-text">Empowering Communities</span>
          <br />
          <span className="text-white">Through Technology</span>
        </h1>
        
        <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10">
          A simple tool for communities to report problems, track progress, and work together to make things better.
        </p>
        
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link to="/report" className="btn-primary flex items-center space-x-2">
            <span>Report an Issue</span>
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link to="/dashboard" className="btn-secondary">
            View Dashboard
          </Link>
        </div>
      </div>

      {/* Features Grid */}
      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 py-16">
        <FeatureCard
          icon={Shield}
          title="Simple Reporting"
          description="Report issues with photos and location. No account required."
        />
        <FeatureCard
          icon={Zap}
          title="Smart Routing"
          description="Issues are routed to the right people automatically."
        />
        <FeatureCard
          icon={Users}
          title="Community Driven"
          description="Everyone can see status updates and track progress."
        />
        <FeatureCard
          icon={BarChart3}
          title="Transparent"
          description="Open data and analytics for everyone to see."
        />
      </div>

      {/* Open Source Section */}
      <div className="glass-panel p-8 rounded-2xl my-16">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-4">Free and Open Source</h2>
          <p className="text-slate-400 max-w-xl mx-auto mb-6">
            Civic Sentinel is licensed under MIT. Anyone can use it, modify it, and deploy it for their community at no cost.
          </p>
          <div className="flex flex-wrap justify-center gap-4">
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">MIT License</span>
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">Self-Hostable</span>
            <span className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm">Community Built</span>
          </div>
        </div>
      </div>
    </div>
  )
}

function FeatureCard({ icon: Icon, title, description }: { icon: React.ElementType; title: string; description: string }) {
  return (
    <div className="glass-panel p-6 hover:border-emerald-500/30 transition-all duration-300">
      <div className="w-12 h-12 rounded-lg bg-emerald-500/10 flex items-center justify-center mb-4">
        <Icon className="w-6 h-6 text-emerald-400" />
      </div>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <p className="text-slate-400 text-sm">{description}</p>
    </div>
  )
}
