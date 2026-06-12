import { useDragStore } from '@/stores/drag-store';

export function DragGhost() {
  const isDragging = useDragStore((s) => s.isDragging);
  const dragLabel = useDragStore((s) => s.dragLabel);
  const dragType = useDragStore((s) => s.dragType);
  const currentX = useDragStore((s) => s.currentX);
  const currentY = useDragStore((s) => s.currentY);

  if (!isDragging || !dragLabel) return null;

  return (
    <div
      style={{
        position: 'fixed',
        left: currentX + 12,
        top: currentY + 12,
        pointerEvents: 'none',
        zIndex: 99999,
      }}
      className="border-accent/20 bg-bg-elevated/90 ring-accent/10 animate-in fade-in zoom-in-95 flex select-none items-center gap-2 rounded-lg border px-3 py-1.5 shadow-2xl ring-1 backdrop-blur-md duration-100"
    >
      <div className="h-2 w-2 animate-pulse rounded-full bg-accent" />
      <span className="text-xs font-semibold text-fg">{dragLabel}</span>
      <span className="rounded bg-border px-1 text-[10px] uppercase tracking-wider text-fg-muted">
        {dragType}
      </span>
    </div>
  );
}
