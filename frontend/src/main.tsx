import React from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import './styles/app.css';
import './styles/query.css';
import './styles/insights.css';

createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
