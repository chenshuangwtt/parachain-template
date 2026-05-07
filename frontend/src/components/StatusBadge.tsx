type StatusBadgeProps = {
  ok: boolean;
  label: string;
};

export function StatusBadge({ ok, label }: StatusBadgeProps) {
  return <span className={`badge ${ok ? "ok" : "warn"}`}>{label}</span>;
}
