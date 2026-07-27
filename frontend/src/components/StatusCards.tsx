import type { StatusResponse } from '../api/client';

type StatusCardsProps = {
  status: StatusResponse | null;
};

export function StatusCards({ status }: StatusCardsProps) {
  const cards = [
    {
      label: 'Service',
      value: status?.service ?? 'Loading',
      hint: 'Rust backend health and state API',
    },
    {
      label: 'Indexing',
      value: status?.indexing_enabled ? 'Enabled' : 'Paused',
      hint: 'Controls repository symbol extraction',
    },
    {
      label: 'Token budget',
      value: status ? status.query_token_budget.toLocaleString() : '—',
      hint: 'Target upper bound for agent-facing summaries',
    },
  ];

  return (
    <section className="status-grid">
      {cards.map((card) => (
        <div className="metric-card" key={card.label}>
          <p>{card.label}</p>
          <strong>{card.value}</strong>
          <small>{card.hint}</small>
        </div>
      ))}
    </section>
  );
}
