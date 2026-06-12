import { useEffect } from 'react';
import { useDragStore, type SplitRegion } from '@/stores/drag-store';
import { usePaneStore } from '@/stores/pane-store';
import { useTabStore } from '@/stores/tab-store';
import { useSshStore } from '@/stores/ssh-store';
import { useHostStore } from '@/stores/host-store';

export function useCustomDragDrop() {
  const isDragging = useDragStore((s) => s.isDragging);

  useEffect(() => {
    if (!isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const elem = document.elementFromPoint(e.clientX, e.clientY);
      let hoveredZoneId: string | null = null;
      let hoveredZoneType: 'pane' | 'empty' | 'tab' | 'tabbar' | null = null;
      let hoveredPaneSplitRegion: SplitRegion | null = null;

      if (elem) {
        const dropZone = elem.closest('[data-drop-zone]');
        if (dropZone) {
          hoveredZoneType = dropZone.getAttribute('data-drop-zone') as
            | 'pane'
            | 'empty'
            | 'tab'
            | 'tabbar'
            | null;
          if (hoveredZoneType === 'pane') {
            hoveredZoneId = dropZone.getAttribute('data-pane-id');
            const rect = dropZone.getBoundingClientRect();
            const relX = (e.clientX - rect.left) / rect.width;
            const relY = (e.clientY - rect.top) / rect.height;

            if (relX < 0.25) hoveredPaneSplitRegion = 'left';
            else if (relX > 0.75) hoveredPaneSplitRegion = 'right';
            else if (relY < 0.25) hoveredPaneSplitRegion = 'top';
            else if (relY > 0.75) hoveredPaneSplitRegion = 'bottom';
            else hoveredPaneSplitRegion = 'center';
          } else if (hoveredZoneType === 'tab') {
            hoveredZoneId = dropZone.getAttribute('data-tab-id');
          }
        }
      }

      useDragStore
        .getState()
        .updateDrag(e.clientX, e.clientY, hoveredZoneId, hoveredZoneType, hoveredPaneSplitRegion);
    };

    const handleMouseUp = (e: MouseEvent) => {
      e.stopPropagation();
      e.preventDefault();

      const state = useDragStore.getState();
      const { dragType, dragId, hoveredZoneId, hoveredZoneType, hoveredPaneSplitRegion } = state;

      if (dragType && dragId) {
        if (dragType === 'host') {
          handleHostDrop(dragId, hoveredZoneType, hoveredZoneId, hoveredPaneSplitRegion);
        } else if (dragType === 'tab') {
          handleTabDrop(dragId, hoveredZoneType, hoveredZoneId, hoveredPaneSplitRegion);
        }
      }

      useDragStore.getState().endDrag();
    };

    window.addEventListener('mousemove', handleMouseMove, true);
    window.addEventListener('mouseup', handleMouseUp, true);

    return () => {
      window.removeEventListener('mousemove', handleMouseMove, true);
      window.removeEventListener('mouseup', handleMouseUp, true);
    };
  }, [isDragging]);
}

function handleHostDrop(
  hostId: string,
  zoneType: string | null,
  zoneId: string | null,
  region: SplitRegion | null,
) {
  const host = useHostStore.getState().hosts.find((h) => h.id === hostId);
  if (!host) return;

  const newTabId = useTabStore.getState().addTab({ label: host.label });

  if (zoneType === 'pane' && zoneId) {
    if (region === 'center') {
      usePaneStore.getState().addTabToPane(zoneId, newTabId);
    } else if (region) {
      usePaneStore.getState().moveTabToNewSplit(newTabId, zoneId, zoneId, region);
    }
  } else {
    // Drop on empty/tabbar — add to active pane
    usePaneStore.getState().ensureTabInPane(newTabId);
  }

  void useSshStore.getState().connectSaved(newTabId, host.id);
}

function handleTabDrop(
  tabId: string,
  zoneType: string | null,
  zoneId: string | null,
  region: SplitRegion | null,
) {
  if (zoneType === 'tab' && zoneId && zoneId !== tabId) {
    // Tab reorder
    const tabs = useTabStore.getState().tabs;
    const fromIndex = tabs.findIndex((t) => t.id === tabId);
    const toIndex = tabs.findIndex((t) => t.id === zoneId);
    if (fromIndex !== -1 && toIndex !== -1) {
      useTabStore.getState().reorderTabs(fromIndex, toIndex);
    }
  } else if (zoneType === 'pane' && zoneId) {
    const sourcePaneId = usePaneStore.getState().getPaneForTab(tabId);
    if (!sourcePaneId) return;

    if (region === 'center') {
      usePaneStore.getState().moveTabToPane(tabId, sourcePaneId, zoneId);
    } else if (region) {
      usePaneStore.getState().moveTabToNewSplit(tabId, sourcePaneId, zoneId, region);
    }
  }
}
