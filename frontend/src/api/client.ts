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
};

export type CreateProjectInput = {
  name: string;
  root_path: string;
};

export type QueryInput = {
  project_id: string | null;
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
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
    ...init,
  });

  if (!response.ok) {
    const body = await response.text();
    let message = body;
    try {
      const parsed = JSON.parse(body) as { message?: string };
      message = parsed.message || body;
    } catch {
      // Keep the raw response body as the error message.
    }
    throw new Error(message || `Request failed: ${response.status}`);
  }

  return response.json() as Promise<T>;
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

export function queryCodeMap(input: QueryInput): Promise<QueryResponse> {
  return request<QueryResponse>('/api/query', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}
