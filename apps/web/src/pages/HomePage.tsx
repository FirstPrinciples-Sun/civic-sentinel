import { Link } from 'react-router-dom'
import { Shield, Zap, Users, BarChart3, ArrowRight, Activity } from 'lucide-react'

export default function HomePage() {
  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      {/* Hero Section */}
      <div className="text-center py-16 lg:py-24">
        <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20 mb-8">
          <Activity className="w-4 h-4 text-emerald-400" />
          <span className="text-sm text-emerald-400 font-medium">Open Source Civic Tech</span>
        </div>
        
        <h1 className="text-4xl md:text-6xl font-bold mb-6">
          <span className="gradient-text">Empowering Communities</span>
          <br />
          <span className="text-white">Through Technology</span>
        </h1>
        
        <p className="text-lg md:text-xl text-slate-400 max-w-2xl mx-auto mb-10">
          Civic Sentinel uses AI to detect, prioritize, and route community issues 
          to the right responders — making every voice heard and every problem solved.
        </p>
        
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <Link to="/report" className="btn-primary flex items-center space-x-2">
            <span>Report an Issue</span>
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link to="/dashboard" className="btn-secondary">
            View Analytics
          </Link>
        </div>
      </div>

      {/* Features Grid */}
      <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6 py-16">
        <FeatureCard
          icon={Shield}
          title="AI Detection"
          description="On-device AI classifies and prioritizes issues automatically using WebAssembly."
        />
        <FeatureCard
          icon={Zap}
          title="Smart Routing"
          description="Issues are automatically routed to the correct department or responder."
        />
        <FeatureCard
          icon={Users}
          title="Community Driven"
          description="Crowdsourced reporting ensures no issue goes unnoticed in your community."
        />
        <FeatureCard
          icon={BarChart3}
          title="Transparent Analytics"
          description="Real-time dashboards show impact metrics and resolution progress."
        />
      </div>

      {/* Stats Section */}
      <div className="glass-panel p-8 rounded-2xl my-16">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
          <StatItem number="0" label="Issues Resolved" />
          <StatItem number="0" label="Communities Served" />
          <StatItem number="0" label="Response Hours" />
          <StatItem number="0" label="Active Responders" />
        </div>
      </div>

      {/* Tech Stack */}
      <div className="py-16 text-center">
        <h2 className="text-2xl font-bold mb-8">Built with Modern Tech Stack</h2>
        <div className="flex flex-wrap justify-center gap-4">
          {['Rust', 'WebAssembly', 'React 18', 'TypeScript', 'Tailwind CSS', 'libSQL'].map((tech) => (
            <span
              key={tech}
              className="px-4 py-2 rounded-lg bg-slate-800 border border-slate-700 text-slate-300 text-sm font-medium"
            >
              {tech}
            </span>
          ))}
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

function StatItem({ number, label }: { number: string; label: string }) {
  return (
    <div>
      <div className="text-3xl md:text-4xl font-bold gradient-text">{number}</div>
      <div className="text-sm text-slate-400 mt-1">{label}</div>
    </div>
  )
}
