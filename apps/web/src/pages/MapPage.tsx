import { MapContainer, TileLayer, Marker, Popup } from 'react-leaflet'

import 'leaflet/dist/leaflet.css'

export default function MapPage() {
  // Default to Bangkok
  const center: [number, number] = [13.7563, 100.5018]

  return (
    <div className="h-[calc(100vh-4rem)] relative">
      <div className="absolute top-4 left-4 z-[1000] glass-panel px-4 py-2">
        <h2 className="text-sm font-semibold">Live Issue Map</h2>
        <p className="text-xs text-slate-400">Real-time community issues</p>
      </div>

      <MapContainer
        center={center}
        zoom={12}
        className="h-full w-full"
        scrollWheelZoom={true}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />
        {/* Sample marker — will be populated from API */}
        <Marker position={center}>
          <Popup>
            <div className="text-slate-900">
              <strong>Welcome to Civic Sentinel</strong>
              <p className="text-sm">Report issues to see them on this map.</p>
            </div>
          </Popup>
        </Marker>
      </MapContainer>
    </div>
  )
}
