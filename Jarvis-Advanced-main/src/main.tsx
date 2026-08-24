import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import App from './App.tsx';
import './index.css';

const originalFetch = window.fetch;
const API_ROOT = ((import.meta as any).env?.VITE_API_URL as string) || "";

if (API_ROOT) {
  window.fetch = function(input: RequestInfo | URL, init?: RequestInit) {
    if (typeof input === "string" && input.startsWith("/api/")) {
      return originalFetch(`${API_ROOT}${input}`, init);
    }
    return originalFetch(input, init);
  };
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
