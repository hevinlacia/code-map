import type { ProjectSummary } from '../api/client';

type ProjectListProps = {
  projects: ProjectSummary[];
  selectedProjectId: string | null;
  scanningProjectId: string | null;
  onSelect: (projectId: string) => void;
  onScan: (projectId: string) => void;
};

export function ProjectList({
  projects,
  selectedProjectId,
  scanningProjectId,
  onSelect,
  onScan,
}: ProjectListProps) {
  return (
    <section className="card">
      <div className="card-header">
        <div>
          <p className="eyebrow">Repositories</p>
          <h2>Indexed workspaces</h2>
        </div>
      </div>

      {projects.length === 0 ? (
        <div className="empty-state">Add a repository path, scan it, then query the code map.</div>
      ) : (
        <div className="project-list">
          {projects.map((project) => {
            const selected = project.id === selectedProjectId;
            const scanning = project.id === scanningProjectId;

            return (
              <article className={selected ? 'project-row selected' : 'project-row'} key={project.id}>
                <button className="project-main" type="button" onClick={() => onSelect(project.id)}>
                  <strong>{project.name}</strong>
                  <code>{project.root_path}</code>
                </button>
                <div className="project-stats">
                  <span className={project.indexed ? 'pill ok' : 'pill muted'}>
                    {project.indexed ? 'Indexed' : 'Not indexed'}
                  </span>
                  <span>{project.repo_count.toLocaleString()} repos</span>
                  <span>{project.file_count.toLocaleString()} files</span>
                  <span>{formatBytes(project.total_bytes)}</span>
                  <button
                    className="secondary small"
                    type="button"
                    disabled={scanning}
                    onClick={() => onScan(project.id)}
                  >
                    {scanning ? 'Scanning…' : 'Scan'}
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
