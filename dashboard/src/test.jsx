import { createRoot } from 'react-dom/client'
import './index.css'

function App() {
  return (
    <div className="flex items-center justify-center min-h-screen bg-slate-950 text-white">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-blue-400">TRM Dashboard</h1>
        <p className="mt-2 text-gray-400">Minimal test — no widgets, no crashes</p>
        <a href="/app.html" className="mt-4 inline-block px-4 py-2 bg-blue-600 rounded hover:bg-blue-700">
          Full Dashboard
        </a>
      </div>
    </div>
  )
}

createRoot(document.getElementById('root')).render(<App />)
