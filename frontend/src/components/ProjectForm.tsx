import { useState, type FormEvent } from 'react';
import type { CreateProjectInput } from '../api/client';

type ProjectFormProps = {
  creating: boolean;
  onCreate: (input: CreateProjectInput) => Promise<void>;
};

export function ProjectForm({ creating, onCreate }: ProjectFormProps) {
  const [name, setName] = useState('');
  const [rootPath, setRootPath] = useState('');

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    await onCreate({ name, root_path: rootPath });
    setName('');
    setRootPath('');
  }

  return (
    <section className="card compact-card">
      <div className="card-header">
        <div>
          <p className="eyebrow">Project</p>
          <h2>Add repository</h2>
        </div>
      </div>

      <form className="project-form" onSubmit={(event) => void handleSubmit(event)}>
        <label>
          <span>Name</span>
          <input
            placeholder="code-map"
            value={name}
            onChange={(event) => setName(event.currentTarget.value)}
          />
        </label>
        <label>
          <span>Root path</span>
          <input
            placeholder="/home/hevin/Developer/tools/code-map"
            value={rootPath}
            onChange={(event) => setRootPath(event.currentTarget.value)}
          />
        </label>
        <button className="primary" type="submit" disabled={creating || !name.trim() || !rootPath.trim()}>
          {creating ? 'Adding…' : 'Add project'}
        </button>
      </form>
    </section>
  );
}
