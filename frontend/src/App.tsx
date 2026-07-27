import { useEffect, useState } from 'react';
import {
  getSettings,
  getStatus,
  listProjects,
  saveSettings,
  type ProjectSummary,
  type RuntimeSettings,
  type StatusResponse,
} from './api/client';
import { ProjectList } from './components/ProjectList';
import { SettingsPanel } from './components/SettingsPanel';
import { StatusCards } from './components/StatusCards';

export function App() {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [settings, setSettings] = useState<RuntimeSettings | null>(null);
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function refresh() {
    try {
      setError(null);
      const [nextStatus, nextSettings, nextProjects] = await Promise.all([
        getStatus(),
        getSettings(),
        listProjects(),
      ]);
      setStatus(nextStatus);
      setSettings(nextSettings);
      setProjects(nextProjects);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to load application state');
    }
  }

  async function handleSave() {
    if (!settings) {
      return;
    }

    try {
      setSaving(true);
      setError(null);
      const nextSettings = await saveSettings(settings);
      setSettings(nextSettings);
      await refresh();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  return (
    <main className="shell">
      <header className="hero">
        <div>
          <p className="eyebrow">Code Map</p>
          <h1>Agent-oriented repository map</h1>
          <p>
            Configure indexing state, summary limits, and workspace status before the symbol graph
            and relationship explorer are implemented.
          </p>
        </div>
        <button className="secondary" type="button" onClick={() => void refresh()}>
          Refresh
        </button>
      </header>

      {error && <div className="alert">{error}</div>}

      <StatusCards status={status} />

      {settings ? (
        <SettingsPanel
          settings={settings}
          saving={saving}
          onChange={setSettings}
          onSave={() => void handleSave()}
        />
      ) : (
        <section className="card">Loading settings…</section>
      )}

      <ProjectList projects={projects} />
    </main>
  );
}
