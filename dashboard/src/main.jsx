import { createRoot } from 'react-dom/client'
import React from 'react'
import './index.css'
import App from './App.jsx'

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props)
    this.state = { hasError: false, error: null }
  }
  static getDerivedStateFromError(error) {
    return { hasError: true, error: error?.message || String(error) }
  }
  componentDidCatch(error, info) {
    console.error('Dashboard Error:', error, info)
  }
  render() {
    if (this.state.hasError) {
      return React.createElement('div', {
        style: { background: '#0a0a0f', color: '#ff4466', padding: '2rem', fontFamily: 'monospace', minHeight: '100vh', whiteSpace: 'pre-wrap' }
      },
        React.createElement('h1', { style: { color: '#ff6688' } }, 'Dashboard Error'),
        React.createElement('div', { style: { color: '#ffaa44', marginTop: '1rem' } }, this.state.error)
      )
    }
    return React.createElement(App)
  }
}

createRoot(document.getElementById('root')).render(React.createElement(ErrorBoundary))
