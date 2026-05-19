import { Routes, Route } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import Layout from './components/Layout'
import HomePage from './pages/HomePage'
import ReportPage from './pages/ReportPage'
import DashboardPage from './pages/DashboardPage'
import MapPage from './pages/MapPage'

function App() {
  return (
    <>
      <Toaster position="top-right" />
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/report" element={<ReportPage />} />
          <Route path="/dashboard" element={<DashboardPage />} />
          <Route path="/map" element={<MapPage />} />
        </Route>
      </Routes>
    </>
  )
}

export default App
