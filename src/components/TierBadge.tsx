interface TierBadgeProps {
  tier: string;
  depth?: number;
}

const tierColors: Record<string, string> = {
  base: "bg-zinc-700 text-zinc-300",
  local: "bg-blue-500/20 text-blue-400",
  development: "bg-green/20 text-green",
  staging: "bg-yellow/20 text-yellow",
  production: "bg-red/20 text-red",
  test: "bg-purple-500/20 text-purple-400",
};

export function TierBadge({ tier, depth }: TierBadgeProps) {
  return (
    <span
      className={`inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium ${
        tierColors[tier] || "bg-surface-2 text-muted"
      }`}
    >
      {tier}
      {depth !== undefined && depth > 0 && (
        <span className="text-[10px] opacity-60">d{depth}</span>
      )}
    </span>
  );
}
