import { create } from 'zustand';

export type DragType = 'host' | 'tab';
export type SplitRegion = 'left' | 'right' | 'top' | 'bottom' | 'center';

interface DragStore {
  isDragging: boolean;
  dragType: DragType | null;
  dragId: string | null;
  dragLabel: string | null;
  currentX: number;
  currentY: number;
  hoveredZoneId: string | null;
  hoveredZoneType: 'pane' | 'empty' | 'tab' | 'tabbar' | null;
  hoveredPaneSplitRegion: SplitRegion | null;

  startDrag: (
    type: DragType,
    id: string,
    label: string,
    x: number,
    y: number,
  ) => void;
  updateDrag: (
    x: number,
    y: number,
    hoveredZoneId: string | null,
    hoveredZoneType: 'pane' | 'empty' | 'tab' | 'tabbar' | null,
    hoveredPaneSplitRegion: SplitRegion | null,
  ) => void;
  endDrag: () => void;
}

export const useDragStore = create<DragStore>((set) => ({
  isDragging: false,
  dragType: null,
  dragId: null,
  dragLabel: null,
  currentX: 0,
  currentY: 0,
  hoveredZoneId: null,
  hoveredZoneType: null,
  hoveredPaneSplitRegion: null,

  startDrag: (type, id, label, x, y) =>
    set({
      isDragging: true,
      dragType: type,
      dragId: id,
      dragLabel: label,
      currentX: x,
      currentY: y,
      hoveredZoneId: null,
      hoveredZoneType: null,
      hoveredPaneSplitRegion: null,
    }),
  updateDrag: (x, y, hoveredZoneId, hoveredZoneType, hoveredPaneSplitRegion) =>
    set({
      currentX: x,
      currentY: y,
      hoveredZoneId,
      hoveredZoneType,
      hoveredPaneSplitRegion,
    }),
  endDrag: () =>
    set({
      isDragging: false,
      dragType: null,
      dragId: null,
      dragLabel: null,
      hoveredZoneId: null,
      hoveredZoneType: null,
      hoveredPaneSplitRegion: null,
    }),
}));
