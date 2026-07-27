import { useEffect, useState } from 'react';
import {
  createProject,
  getSettings,
  getStatus,
  listProjects,
  queryCodeMap,
  saveSettings,
  scanProject,
  type CreateProjectInput,
  type ProjectSummary,
  type QueryResponse,
  type RuntimeSettings,
  type StatusResponse,
} from './api/client';
import { ProjectForm } from './components/ProjectForm';
import { ProjectList } from './components/ProjectList';
import { QueryPanel } from './components/QueryPanel';
import { SettingsPanel } from './components/SettingsPanel';
import { StatusCards } from './components/StatusCards';

export function App() {
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [settings, setSettings] = useState<RuntimeSettings | null>(null);
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [queryResult, setQueryResult] = useState<QueryResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [creating, setCreating] = useState(false);
  const [scanningProjectId, setScanningProjectId] = useState<string | null>(null);
  const [querying, setQuerying] = useState(false);

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
      setSelectedProjectId((current) => {
        if (current && nextProjects.some((project) => project.id === current)) {
          return current;
        }
        return nextSettings.active_project_id ?? nextProjects[0]?.id ?? null;
      });
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

  async function handleCreateProject(input: CreateProjectInput) {
    try {
      setCreating(true);
      setError(null);
      const created = await createProject(input);
      setSelectedProjectId(created.id);
      await refresh();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to create project');
    } finally {
      setCreating(false);
    }
  }

  async function handleScanProject(projectId: string) {
    try {
      setScanningProjectId(projectId);
      setError(null);
      const scanned = await scanProject(projectId);
      setProjects((current) =>
        current.map((project) => (project.id === scanned.id ? scanned : project)),
      );
      setSelectedProjectId(projectId);
      setQueryResult(null);
      await refresh();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to scan project');
    } finally {
      setScanningProjectId(null);
    }
  }

  async function handleQuery(query: string, maxResults: number) {
    try {
      setQuerying(true);
      setError(null);
      const result = await queryCodeMap({
        project_id: selectedProjectId,
        query,
        max_results: maxResults,
      });
      setQueryResult(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Failed to query code map');
    } finally {
      setQuerying(false);
    }
  }

  function handleSelectProject(projectId: string) {
    setSelectedProjectId(projectId);
    setQueryResult(null);
    if (settings) {
      const nextSettings = { ...settings, active_project_id: projectId };
      setSettings(nextSettings);
      void saveSettings(nextSettings).catch((cause) => {
        setError(cause instanceof Error ? cause.message : 'Failed to save active project');
      });
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
            Add a repository, scan it, then query keywords, endpoints, config keys, or class names
            to get compact file and line suggestions for coding agents.
          </p>
        </div>
        <button className="secondary" type="button" onClick={() => void refresh()}>
          Refresh
        </button>
      </header>

      {error && <div className="alert">{error}</div>}

      <StatusCards status={status} />

      <div className="two-column">
        <ProjectForm creating={creating} onCreate={handleCreateProject} />
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
      </div>

      <ProjectList
        projects={projects}
        selectedProjectId={selectedProjectId}
        scanningProjectId={scanningProjectId}
        onSelect={handleSelectProject}
        onScan={(projectId) => void handleScanProject(projectId)}
      />

      <QueryPanel
        projects={projects}
        selectedProjectId={selectedProjectId}
        querying={querying}
        queryResult={queryResult}
        onSelectProject={handleSelectProject}
        onQuery={handleQuery}
      />
    </main>
  );
}
