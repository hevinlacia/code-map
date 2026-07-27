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
  symbol_count: number;
  relationship_count: number;
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
    throw new Error(body || `Request failed: ${response.status}`);
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
