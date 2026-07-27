import type { RuntimeSettings } from '../api/client';

type SettingsPanelProps = {
  settings: RuntimeSettings;
  saving: boolean;
  onChange: (settings: RuntimeSettings) => void;
  onSave: () => void;
};

export function SettingsPanel({ settings, saving, onChange, onSave }: SettingsPanelProps) {
  return (
    <section className="card">
      <div className="card-header">
        <div>
          <p className="eyebrow">Configuration</p>
          <h2>Agent context controls</h2>
        </div>
        <button className="primary" type="button" disabled={saving} onClick={onSave}>
          {saving ? 'Saving…' : 'Save settings'}
        </button>
      </div>

      <div className="settings-grid">
        <label className="toggle-row">
          <span>
            <strong>Indexing enabled</strong>
            <small>Allow background project indexing and symbol extraction.</small>
          </span>
          <input
            type="checkbox"
            checked={settings.indexing_enabled}
            onChange={(event) =>
              onChange({ ...settings, indexing_enabled: event.currentTarget.checked })
            }
          />
        </label>

        <label className="toggle-row">
          <span>
            <strong>Auto refresh</strong>
            <small>Refresh indexes automatically when write actions are enabled.</small>
          </span>
          <input
            type="checkbox"
            checked={settings.auto_refresh_enabled}
            onChange={(event) =>
              onChange({ ...settings, auto_refresh_enabled: event.currentTarget.checked })
            }
          />
        </label>

        <label>
          <span>Query token budget</span>
          <input
            type="number"
            min={200}
            step={100}
            value={settings.query_token_budget}
            onChange={(event) =>
              onChange({ ...settings, query_token_budget: Number(event.currentTarget.value) })
            }
          />
        </label>

        <label>
          <span>Max summary lines</span>
          <input
            type="number"
            min={1}
            max={500}
            value={settings.max_summary_lines}
            onChange={(event) =>
              onChange({ ...settings, max_summary_lines: Number(event.currentTarget.value) })
            }
          />
        </label>
      </div>
    </section>
  );
}
