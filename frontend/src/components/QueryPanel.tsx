import { useState, type FormEvent } from 'react';
import type { ProjectSummary, QueryResponse } from '../api/client';

type QueryPanelProps = {
  projects: ProjectSummary[];
  selectedProjectId: string | null;
  querying: boolean;
  queryResult: QueryResponse | null;
  onSelectProject: (projectId: string) => void;
  onQuery: (query: string, maxResults: number) => Promise<void>;
};

export function QueryPanel({
  projects,
  selectedProjectId,
  querying,
  queryResult,
  onSelectProject,
  onQuery,
}: QueryPanelProps) {
  const [query, setQuery] = useState('');
  const [maxResults, setMaxResults] = useState(12);
  const selectedProject = projects.find((project) => project.id === selectedProjectId);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onQuery(query, maxResults);
  }

  return (
    <section className="card">
      <div className="card-header">
        <div>
          <p className="eyebrow">Agent Query</p>
          <h2>Find relevant files and lines</h2>
        </div>
      </div>

      <form className="query-form" onSubmit={(event) => void handleSubmit(event)}>
        <label>
          <span>Project</span>
          <select
            value={selectedProjectId ?? ''}
            onChange={(event) => onSelectProject(event.currentTarget.value)}
          >
            <option value="" disabled>
              Select a scanned project
            </option>
            {projects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name} {project.indexed ? '' : '(not scanned)'}
              </option>
            ))}
          </select>
        </label>

        <label className="query-input">
          <span>Keyword / endpoint / class / config key</span>
          <input
            placeholder="settings service, /api/projects, RuntimeState, CODE_MAP_FRONTEND_DIR"
            value={query}
            onChange={(event) => setQuery(event.currentTarget.value)}
          />
        </label>

        <label className="narrow-input">
          <span>Results</span>
          <input
            type="number"
            min={1}
            max={50}
            value={maxResults}
            onChange={(event) => setMaxResults(Number(event.currentTarget.value))}
          />
        </label>

        <button
          className="primary"
          type="submit"
          disabled={querying || !query.trim() || !selectedProject || !selectedProject.indexed}
        >
          {querying ? 'Querying…' : 'Query'}
        </button>
      </form>

      {selectedProject && !selectedProject.indexed && (
        <div className="hint">Scan this project before querying.</div>
      )}

      {queryResult && (
        <div className="query-results">
          <div className="summary-box">
            {queryResult.summary_lines.map((line) => (
              <p key={line}>{line}</p>
            ))}
          </div>

          <div className="result-list">
            {queryResult.results.map((result) => (
              <article className="result-row" key={result.relative_path}>
                <div className="result-heading">
                  <code>{formatResultPath(result)}</code>
                  <span className="score">score {result.score}</span>
                </div>
                <div className="reason-list">
                  {result.reasons.map((reason) => (
                    <span key={reason}>{reason}</span>
                  ))}
                </div>
                {result.symbols.length > 0 && (
                  <div className="insight-list">
                    <strong>Symbols</strong>
                    {result.symbols.map((symbol) => (
                      <span key={`symbol:${symbol.kind}:${symbol.name}:${symbol.line}`}>
                        {symbol.kind} · {symbol.name} · L{symbol.line}
                      </span>
                    ))}
                  </div>
                )}
                {result.relationships.length > 0 && (
                  <div className="insight-list">
                    <strong>Relationships</strong>
                    {result.relationships.map((relationship) => (
                      <span key={`rel:${relationship.kind}:${relationship.to}:${relationship.line}`}>
                        {relationship.kind} · {relationship.from} → {relationship.to} · L{relationship.line}
                      </span>
                    ))}
                  </div>
                )}
                {result.snippets.length > 0 && (
                  <div className="snippet-list">
                    {result.snippets.map((snippet) => (
                      <pre key={`${result.relative_path}:${snippet.line}`}>
                        <span>{snippet.line}</span> {snippet.text}
                      </pre>
                    ))}
                  </div>
                )}
              </article>
            ))}
          </div>
        </div>
      )}
    </section>
  );
}

function formatResultPath(result: { repo: string | null; repo_relative_path: string; relative_path: string }): string {
  return result.repo ? `${result.repo}:${result.repo_relative_path}` : result.relative_path;
}
