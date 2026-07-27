import type { ProjectSummary } from '../api/client';

type ProjectListProps = {
  projects: ProjectSummary[];
};

export function ProjectList({ projects }: ProjectListProps) {
  return (
    <section className="card">
      <div className="card-header">
        <div>
          <p className="eyebrow">Repositories</p>
          <h2>Indexed workspaces</h2>
        </div>
        <button className="secondary" type="button" disabled>
          Add project soon
        </button>
      </div>

      <div className="project-list">
        {projects.map((project) => (
          <article className="project-row" key={project.id}>
            <div>
              <strong>{project.name}</strong>
              <code>{project.root_path}</code>
            </div>
            <div className="project-stats">
              <span className={project.indexed ? 'pill ok' : 'pill muted'}>
                {project.indexed ? 'Indexed' : 'Not indexed'}
              </span>
              <span>{project.symbol_count.toLocaleString()} symbols</span>
              <span>{project.relationship_count.toLocaleString()} links</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}
