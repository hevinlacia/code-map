export type RuntimeSettings = {
  indexing_enabled: boolean;
  auto_refresh_enabled: boolean;
  active_project_id: string | null;
  query_token_budget: number;
  max_summary_lines: number;
};

export type StatusResponse = RuntimeSettings & {
  service: string;
};

export type GitSnapshot = {
  branch: string | null;
  head: string | null;
  dirty: boolean;
  repo_count: number;
  dirty_repo_count: number;
  fingerprint: string | null;
};

export type IndexFreshness = {
  current_git: GitSnapshot | null;
  last_scanned_git: GitSnapshot | null;
  index_stale: boolean;
  auto_refreshed: boolean;
  warning: string | null;
};

export type ProjectKind = 'baseline' | 'branch_overlay';

export type ProjectSummary = {
  id: string;
  name: string;
  root_path: string;
  indexed: boolean;
  last_indexed_at: string | null;
  file_count: number;
  repo_count: number;
  total_bytes: number;
  symbol_count: number;
  relationship_count: number;
  root_exists: boolean;
  worktree_project: boolean;
  kind: ProjectKind;
  base_project_id: string | null;
  branch_name: string | null;
  baseline_branch: string | null;
  overlay_file_count: number;
  overlay_deleted_count: number;
  last_used_at: string | null;
  current_git: GitSnapshot | null;
  last_scanned_git: GitSnapshot | null;
  index_stale: boolean;
};

export type CreateProjectInput = {
  name: string;
  root_path: string;
  kind?: ProjectKind;
  base_project_id?: string | null;
  branch_name?: string | null;
  baseline_branch?: string | null;
};

export type QueryInput = {
  project_id: string | null;
  branch_name?: string | null;
  query: string;
  max_results?: number;
};

export type QuerySnippet = {
  line: number;
  text: string;
};

export type QuerySymbol = {
  kind: string;
  name: string;
  detail: string | null;
  relative_path: string;
  repo: string | null;
  repo_relative_path: string;
  line: number;
};

export type QueryRelationship = {
  kind: string;
  from: string;
  to: string;
  detail: string | null;
  relative_path: string;
  repo: string | null;
  repo_relative_path: string;
  line: number;
};

export type QueryResult = {
  relative_path: string;
  repo: string | null;
  repo_relative_path: string;
  language: string | null;
  score: number;
  reasons: string[];
  snippets: QuerySnippet[];
  symbols: QuerySymbol[];
  relationships: QueryRelationship[];
};

export type QueryResponse = {
  project_id: string;
  project_name: string;
  query: string;
  terms: string[];
  result_count: number;
  summary_lines: string[];
  results: QueryResult[];
  index_freshness: IndexFreshness;
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
    ...init,
  });

  const body = await response.text();

  if (!response.ok) {
    let message = body;
    try {
      const parsed = JSON.parse(body) as { message?: string };
      message = parsed.message || body;
    } catch {
      // Keep the raw response body as the error message.
    }
    throw new Error(message || `Request failed: ${response.status}`);
  }

  if (!body) {
    return undefined as T;
  }

  return JSON.parse(body) as T;
}

export function getStatus(): Promise<StatusResponse> {
  return request<StatusResponse>('/api/status');
}

export function getSettings(): Promise<RuntimeSettings> {
  return request<RuntimeSettings>('/api/settings');
}

export function saveSettings(settings: RuntimeSettings): Promise<RuntimeSettings> {
  return request<RuntimeSettings>('/api/settings', {
    method: 'PUT',
    body: JSON.stringify(settings),
  });
}

export function listProjects(): Promise<ProjectSummary[]> {
  return request<ProjectSummary[]>('/api/projects');
}

export function createProject(input: CreateProjectInput): Promise<ProjectSummary> {
  return request<ProjectSummary>('/api/projects', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}

export function scanProject(projectId: string): Promise<ProjectSummary> {
  return request<ProjectSummary>(`/api/projects/${projectId}/scan`, {
    method: 'POST',
  });
}

export function deleteProject(projectId: string): Promise<void> {
  return request<void>(`/api/projects/${projectId}`, {
    method: 'DELETE',
  });
}

export function queryCodeMap(input: QueryInput): Promise<QueryResponse> {
  return request<QueryResponse>('/api/query', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}
