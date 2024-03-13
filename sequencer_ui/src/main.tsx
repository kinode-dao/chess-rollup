import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import { Web3ContextProvider } from './libs/components/Web3ContextProvider'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Web3ContextProvider>
      <App />
    </Web3ContextProvider>
  </React.StrictMode>,
)
