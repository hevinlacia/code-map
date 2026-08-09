import type { ProjectSummary } from '../api/client';

type ProjectListProps = {
  projects: ProjectSummary[];
  selectedProjectId: string | null;
  scanningProjectId: string | null;
  deletingProjectId: string | null;
  onSelect: (projectId: string) => void;
  onScan: (projectId: string) => void;
  onDelete: (projectId: string) => void;
};

export function ProjectList({
  projects,
  selectedProjectId,
  scanningProjectId,
  deletingProjectId,
  onSelect,
  onScan,
  onDelete,
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
            const deleting = project.id === deletingProjectId;

            return (
              <article className={selected ? 'project-row selected' : 'project-row'} key={project.id}>
                <button className="project-main" type="button" onClick={() => onSelect(project.id)}>
                  <strong>{project.name}</strong>
                  <code>{project.root_path}</code>
                  <small className="project-meta">{formatGitSummary(project)}</small>
                </button>
                <div className="project-stats">
                  <span className={project.kind === 'baseline' ? 'pill ok' : 'pill info'}>
                    {project.kind === 'baseline' ? 'Baseline' : 'Overlay'}
                  </span>
                  <span className={project.indexed ? 'pill ok' : 'pill muted'}>
                    {project.indexed ? 'Indexed' : 'Not indexed'}
                  </span>
                  <span className={!project.root_exists ? 'pill danger' : project.index_stale ? 'pill warn' : 'pill ok'}>
                    {!project.root_exists ? 'Missing' : project.index_stale ? 'Stale' : 'Fresh'}
                  </span>
                  <span>{project.repo_count.toLocaleString()} repos</span>
                  <span>{project.kind === 'baseline' ? `${project.file_count.toLocaleString()} files` : `${project.overlay_file_count.toLocaleString()} changed / ${project.overlay_deleted_count.toLocaleString()} deleted`}</span>
                  <span>{formatBytes(project.total_bytes)}</span>
                  <button
                    className="secondary small"
                    type="button"
                    disabled={scanning || deleting || !project.root_exists}
                    onClick={() => onScan(project.id)}
                  >
                    {scanning ? 'Scanning…' : 'Scan'}
                  </button>
                  <button
                    className="secondary small danger-button"
                    type="button"
                    disabled={scanning || deleting}
                    onClick={() => onDelete(project.id)}
                  >
                    {deleting ? 'Removing…' : 'Remove index'}
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

function formatGitSummary(project: ProjectSummary): string {
  if (!project.root_exists) {
    return 'Root path missing; remove this index or restore the worktree';
  }

  const current = project.current_git;
  const scanned = project.last_scanned_git;
  if (!current) {
    return 'No Git metadata detected';
  }

  const parts = [
    project.kind === 'baseline' ? 'baseline' : 'overlay',
    current.branch ? `branch ${current.branch}` : 'mixed branches',
    current.head ? `head ${current.head.slice(0, 8)}` : 'mixed heads',
  ];
  if (current.dirty) {
    parts.push(`${current.dirty_repo_count} dirty repo${current.dirty_repo_count === 1 ? '' : 's'}`);
  }
  if (project.index_stale && scanned?.head) {
    parts.push(`indexed ${scanned.head.slice(0, 8)}`);
  }
  return parts.join(' · ');
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
