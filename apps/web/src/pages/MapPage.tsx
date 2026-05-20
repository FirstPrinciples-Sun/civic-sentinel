import { useState, useEffect } from 'react'
import { MapContainer, TileLayer, Marker, Popup, useMap } from 'react-leaflet'
import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import { useIssues } from '../hooks/useIssues'
import {
  Loader2,
  Search,
  MapPin,
  Calendar,
  ChevronLeft,
  ChevronRight,
  Info,
} from 'lucide-react'
import { format } from 'date-fns'

// Custom marker status colors
const getStatusColor = (status: string) => {
  switch (status.toLowerCase()) {
    case 'reported':
      return '#f59e0b' // Amber
    case 'underreview':
      return '#3b82f6' // Blue
    case 'inprogress':
      return '#8b5cf6' // Purple
    case 'resolved':
      return '#10b981' // Green
    case 'closed':
      return '#64748b' // Slate
    case 'escalated':
      return '#ef4444' // Red
    default:
      return '#10b981'
  }
}

const getStatusBadgeClass = (status: string) => {
  switch (status.toLowerCase()) {
    case 'reported':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'underreview':
      return 'text-blue-400 bg-blue-500/10 border-blue-500/20'
    case 'inprogress':
      return 'text-purple-400 bg-purple-500/10 border-purple-500/20'
    case 'resolved':
      return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20'
    case 'closed':
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
    case 'escalated':
      return 'text-red-400 bg-red-500/10 border-red-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
  }
}

const getPriorityBadgeClass = (priority: string) => {
  switch (priority.toLowerCase()) {
    case 'low':
      return 'text-slate-400 bg-slate-500/10 border-slate-500/20'
    case 'medium':
      return 'text-amber-400 bg-amber-500/10 border-amber-500/20'
    case 'high':
      return 'text-orange-400 bg-orange-500/10 border-orange-500/20'
    case 'critical':
      return 'text-red-400 bg-red-500/10 border-red-500/20'
    default:
      return 'text-slate-400 bg-slate-500/10'
  }
}

const createCustomIcon = (status: string) => {
  const color = getStatusColor(status)
  return L.divIcon({
    html: `
      <div class="relative w-8 h-8 flex items-center justify-center">
        <div class="absolute w-8 h-8 rounded-full opacity-35 animate-ping" style="background-color: ${color}"></div>
        <svg class="w-8 h-8 filter drop-shadow-md z-10" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2C8.13 2 5 5.13 5 9C5 14.25 12 22 12 22C12 22 19 14.25 19 9C19 5.13 15.87 2 12 2ZM12 11.5C10.62 11.5 9.5 10.38 9.5 9C9.5 7.62 10.62 6.5 12 6.5C13.38 6.5 14.5 7.62 14.5 9C14.5 10.38 13.38 11.5 12 11.5Z" fill="${color}"/>
        </svg>
      </div>
    `,
    className: 'custom-marker-icon',
    iconSize: [32, 32],
    iconAnchor: [16, 32],
    popupAnchor: [0, -32],
  })
}

function MapController({ center }: { center: [number, number] | null }) {
  const map = useMap()
  useEffect(() => {
    if (center) {
      map.setView(center, 15, { animate: true, duration: 1.5 })
    }
  }, [center, map])
  return null
}

export default function MapPage() {
  const { data: issues, isLoading } = useIssues(1, 100)
  const [selectedIssue, setSelectedIssue] = useState<any | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [statusFilter, setStatusFilter] = useState('all')
  const [categoryFilter, setCategoryFilter] = useState('all')
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)

  const center: [number, number] = [13.7563, 100.5018] // Bangkok default

  const filteredIssues =
    issues?.filter((issue) => {
      const matchesSearch =
        issue.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        issue.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (issue.location.address &&
          issue.location.address.toLowerCase().includes(searchTerm.toLowerCase()))
      const matchesStatus =
        statusFilter === 'all' || issue.status.toLowerCase() === statusFilter.toLowerCase()
      const matchesCategory =
        categoryFilter === 'all' || issue.category.toLowerCase() === categoryFilter.toLowerCase()
      return matchesSearch && matchesStatus && matchesCategory
    }) ?? []

  const activeCenter: [number, number] | null = selectedIssue
    ? [selectedIssue.location.latitude, selectedIssue.location.longitude]
    : null

  return (
    <div className="h-[calc(100vh-4rem)] relative overflow-hidden bg-slate-950">
      {/* Map Layer */}
      <MapContainer
        center={center}
        zoom={12}
        className="h-full w-full z-10"
        scrollWheelZoom={true}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        <MapController center={activeCenter} />

        {filteredIssues.map((issue) => (
          <Marker
            key={issue.id}
            position={[issue.location.latitude, issue.location.longitude]}
            icon={createCustomIcon(issue.status)}
            eventHandlers={{
              click: () => {
                setSelectedIssue(issue)
              },
            }}
          >
            <Popup>
              <div className="p-1 max-w-sm text-slate-800 font-sans">
                <h3 className="font-bold text-base mb-1 leading-tight text-slate-900">
                  {issue.title}
                </h3>
                <div className="flex flex-wrap gap-1.5 mb-2">
                  <span className={`px-2 py-0.5 text-xs font-semibold rounded border ${getStatusBadgeClass(issue.status)}`}>
                    {issue.status.toUpperCase()}
                  </span>
                  <span className={`px-2 py-0.5 text-xs font-semibold rounded border ${getPriorityBadgeClass(issue.priority)}`}>
                    {issue.priority.toUpperCase()}
                  </span>
                </div>
                <p className="text-sm text-slate-600 mb-3 line-clamp-3">
                  {issue.description}
                </p>
                {issue.location.address && (
                  <div className="flex items-center gap-1 text-xs text-slate-500 border-t pt-2 border-slate-200">
                    <MapPin className="w-3.5 h-3.5 flex-shrink-0 text-slate-400" />
                    <span className="truncate">{issue.location.address}</span>
                  </div>
                )}
              </div>
            </Popup>
          </Marker>
        ))}
      </MapContainer>

      {/* Sidebar Controls overlay */}
      <div
        className={`absolute top-4 left-4 z-[1000] max-h-[calc(100vh-6rem)] w-80 glass-panel flex flex-col pointer-events-auto transition-all duration-300 shadow-2xl overflow-hidden border-slate-700/60 bg-slate-900/85 backdrop-blur-xl ${
          isSidebarOpen ? 'translate-x-0' : '-translate-x-[calc(100%+1rem)]'
        }`}
      >
        {/* Toggle Button */}
        <button
          onClick={() => setIsSidebarOpen(!isSidebarOpen)}
          className="absolute -right-10 top-4 w-10 h-10 bg-slate-800 border-r border-y border-slate-700/50 rounded-r-lg flex items-center justify-center shadow-lg text-emerald-400 hover:text-emerald-300 transition-colors pointer-events-auto"
        >
          {isSidebarOpen ? <ChevronLeft className="w-5 h-5" /> : <ChevronRight className="w-5 h-5" />}
        </button>

        <div className="p-4 border-b border-slate-700/50">
          <h2 className="text-lg font-bold gradient-text mb-1">Live Issue Map</h2>
          <p className="text-xs text-slate-400">Filter and track active public issues</p>
        </div>

        {/* Filters */}
        <div className="p-4 space-y-3 border-b border-slate-700/50 bg-slate-800/20">
          <div className="relative">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search issues or address..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="w-full pl-9 pr-4 py-2 bg-slate-800/80 border border-slate-700/50 rounded-lg text-sm placeholder-slate-500 focus:outline-none focus:border-emerald-500/50 transition-colors text-white"
            />
          </div>

          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-[10px] font-semibold text-slate-400 mb-1 uppercase tracking-wider">Status</label>
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="w-full px-2 py-1.5 bg-slate-800/80 border border-slate-700/50 rounded-lg text-xs focus:outline-none focus:border-emerald-500/50 transition-colors text-white"
              >
                <option value="all">All</option>
                <option value="reported">Reported</option>
                <option value="underreview">Reviewing</option>
                <option value="inprogress">In Progress</option>
                <option value="resolved">Resolved</option>
                <option value="closed">Closed</option>
                <option value="escalated">Escalated</option>
              </select>
            </div>

            <div>
              <label className="block text-[10px] font-semibold text-slate-400 mb-1 uppercase tracking-wider">Category</label>
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                className="w-full px-2 py-1.5 bg-slate-800/80 border border-slate-700/50 rounded-lg text-xs focus:outline-none focus:border-emerald-500/50 transition-colors text-white"
              >
                <option value="all">All</option>
                <option value="infrastructure">Infrastructure</option>
                <option value="utilities">Utilities</option>
                <option value="sanitation">Sanitation</option>
                <option value="safety">Public Safety</option>
                <option value="environment">Environment</option>
                <option value="other">Other</option>
              </select>
            </div>
          </div>
        </div>

        {/* Scrollable list */}
        <div className="flex-1 overflow-y-auto divide-y divide-slate-800/60 custom-scrollbar">
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 text-slate-400">
              <Loader2 className="w-8 h-8 animate-spin text-emerald-400 mb-2" />
              <span className="text-xs">Loading issues...</span>
            </div>
          ) : filteredIssues.length === 0 ? (
            <div className="text-center py-12 text-slate-500 px-4">
              <Info className="w-6 h-6 mx-auto mb-2 text-slate-600" />
              <p className="text-xs">No matching issues found on the map.</p>
            </div>
          ) : (
            filteredIssues.map((issue) => (
              <div
                key={issue.id}
                onClick={() => setSelectedIssue(issue)}
                className={`p-3 text-left transition-all duration-150 cursor-pointer ${
                  selectedIssue?.id === issue.id
                    ? 'bg-emerald-500/10 border-l-2 border-emerald-500'
                    : 'hover:bg-slate-800/35 border-l-2 border-transparent'
                }`}
              >
                <h4 className="font-semibold text-sm text-slate-100 truncate mb-1">
                  {issue.title}
                </h4>
                <p className="text-xs text-slate-400 line-clamp-2 mb-2">
                  {issue.description}
                </p>
                <div className="flex items-center justify-between">
                  <span className={`px-1.5 py-0.5 text-[10px] font-bold rounded border ${getStatusBadgeClass(issue.status)}`}>
                    {issue.status.toUpperCase()}
                  </span>
                  <span className="text-[10px] text-slate-500 flex items-center gap-0.5">
                    <Calendar className="w-3 h-3" />
                    {format(new Date(issue.created_at), 'MMM dd')}
                  </span>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Floating Toggle Button (visible only when sidebar is closed) */}
      {!isSidebarOpen && (
        <button
          onClick={() => setIsSidebarOpen(true)}
          className="absolute top-4 left-4 z-[1000] w-10 h-10 bg-slate-800/90 border border-slate-700/50 rounded-lg flex items-center justify-center shadow-lg text-emerald-400 hover:text-emerald-300 pointer-events-auto"
        >
          <ChevronRight className="w-5 h-5" />
        </button>
      )}
    </div>
  )
}
