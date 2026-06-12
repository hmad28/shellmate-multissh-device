import { create } from 'zustand';

export type SplitDirection = 'horizontal' | 'vertical';
export type DropPosition = 'left' | 'right' | 'top' | 'bottom' | 'center';

export interface LeafNode {
  type: 'leaf';
  id: string;
  tabIds: string[];
  activeTabId: string | null;
}

export interface SplitNode {
  type: 'split';
  id: string;
  direction: SplitDirection;
  children: PaneNode[];
  sizes: number[];
}

export type PaneNode = LeafNode | SplitNode;

// --- Tree helpers ---

let nextId = 1;
const genId = () => `pane-${nextId++}`;

function createLeaf(
  tabIds: string[] = [],
  activeTabId: string | null = null,
): LeafNode {
  return { type: 'leaf', id: genId(), tabIds, activeTabId };
}

export function findNode(root: PaneNode, id: string): PaneNode | null {
  if (root.id === id) return root;
  if (root.type === 'split') {
    for (const child of root.children) {
      const found = findNode(child, id);
      if (found) return found;
    }
  }
  return null;
}

function findParentSplit(
  root: PaneNode,
  childId: string,
): SplitNode | null {
  if (root.type === 'split') {
    for (const child of root.children) {
      if (child.id === childId) return root;
      const found = findParentSplit(child, childId);
      if (found) return found;
    }
  }
  return null;
}

export function getAllLeaves(root: PaneNode): LeafNode[] {
  if (root.type === 'leaf') return [root];
  return root.children.flatMap(getAllLeaves);
}

function findLeafContainingTab(
  root: PaneNode,
  tabId: string,
): LeafNode | null {
  if (root.type === 'leaf') {
    return root.tabIds.includes(tabId) ? root : null;
  }
  for (const child of root.children) {
    const found = findLeafContainingTab(child, tabId);
    if (found) return found;
  }
  return null;
}

function replaceNode(
  root: PaneNode,
  targetId: string,
  replacement: PaneNode,
): PaneNode {
  if (root.id === targetId) return replacement;
  if (root.type === 'split') {
    return {
      ...root,
      children: root.children.map((c) => replaceNode(c, targetId, replacement)),
    };
  }
  return root;
}

function removeNode(root: PaneNode, targetId: string): PaneNode | null {
  if (root.id === targetId) return null;
  if (root.type === 'split') {
    const surviving = root.children
      .map((child, i) => ({ node: removeNode(child, targetId), idx: i }))
      .filter((e): e is { node: PaneNode; idx: number } => e.node !== null);

    if (surviving.length === 0) return null;
    if (surviving.length === 1) return surviving[0]!.node;

    const sizes = surviving.map((e) => root.sizes[e.idx] ?? 50);
    const total = sizes.reduce((a, b) => a + b, 0);
    return {
      ...root,
      children: surviving.map((e) => e.node),
      sizes: sizes.map((s) => (s / total) * 100),
    };
  }
  return root;
}

function updateLeaf(
  root: PaneNode,
  leafId: string,
  updater: (leaf: LeafNode) => LeafNode,
): PaneNode {
  if (root.type === 'leaf' && root.id === leafId) return updater(root);
  if (root.type === 'split') {
    return {
      ...root,
      children: root.children.map((c) => updateLeaf(c, leafId, updater)),
    };
  }
  return root;
}

// --- Store ---

interface PaneStore {
  root: PaneNode;
  activePaneId: string;
  fullscreenPaneId: string | null;

  ensureTabInPane: (tabId: string) => void;
  setActivePane: (paneId: string) => void;
  setActiveTabInPane: (paneId: string, tabId: string) => void;
  addTabToPane: (paneId: string, tabId: string) => void;
  moveTabToPane: (tabId: string, sourcePaneId: string, targetPaneId: string) => void;
  moveTabToNewSplit: (
    tabId: string,
    sourcePaneId: string,
    targetPaneId: string,
    position: DropPosition,
  ) => void;
  splitPane: (
    targetPaneId: string,
    direction: SplitDirection,
    position: 'before' | 'after',
    tabId: string,
  ) => void;
  closeTabInPane: (paneId: string, tabId: string) => void;
  collapsePane: (paneId: string) => void;
  toggleFullscreen: (paneId: string) => void;
  updateSizes: (splitId: string, sizes: number[]) => void;
  getPaneForTab: (tabId: string) => string | null;
}

export const usePaneStore = create<PaneStore>((set, get) => {
  const initialLeaf = createLeaf();
  return {
    root: initialLeaf,
    activePaneId: initialLeaf.id,
    fullscreenPaneId: null,

    ensureTabInPane: (tabId) => {
      const { root } = get();
      if (findLeafContainingTab(root, tabId)) return;

      // Add to active pane or first leaf
      const { activePaneId } = get();
      const target = findNode(root, activePaneId);
      const paneId =
        target && target.type === 'leaf'
          ? activePaneId
          : (getAllLeaves(root)[0]?.id ?? activePaneId);

      set({
        root: updateLeaf(root, paneId, (leaf) => ({
          ...leaf,
          tabIds: [...leaf.tabIds, tabId],
          activeTabId: tabId,
        })),
      });
    },

    setActivePane: (paneId) => set({ activePaneId: paneId }),

    setActiveTabInPane: (paneId, tabId) => {
      set((s) => ({
        root: updateLeaf(s.root, paneId, (leaf) => ({
          ...leaf,
          activeTabId: tabId,
        })),
        activePaneId: paneId,
      }));
    },

    addTabToPane: (paneId, tabId) => {
      set((s) => ({
        root: updateLeaf(s.root, paneId, (leaf) => ({
          ...leaf,
          tabIds: [...leaf.tabIds, tabId],
          activeTabId: tabId,
        })),
        activePaneId: paneId,
      }));
    },

    moveTabToPane: (tabId, sourcePaneId, targetPaneId) => {
      if (sourcePaneId === targetPaneId) return;
      const { root } = get();
      // Remove from source
      let newRoot = updateLeaf(root, sourcePaneId, (leaf) => {
        const newTabIds = leaf.tabIds.filter((t) => t !== tabId);
        return {
          ...leaf,
          tabIds: newTabIds,
          activeTabId:
            leaf.activeTabId === tabId
              ? (newTabIds[0] ?? null)
              : leaf.activeTabId,
        };
      });
      // Add to target
      newRoot = updateLeaf(newRoot, targetPaneId, (leaf) => ({
        ...leaf,
        tabIds: [...leaf.tabIds, tabId],
        activeTabId: tabId,
      }));
      // Collapse empty source
      const src = findNode(newRoot, sourcePaneId);
      if (src && src.type === 'leaf' && src.tabIds.length === 0) {
        const leaves = getAllLeaves(newRoot);
        if (leaves.length > 1) {
          newRoot = removeNode(newRoot, sourcePaneId) ?? newRoot;
        }
      }
      set({ root: newRoot, activePaneId: targetPaneId });
    },

    moveTabToNewSplit: (tabId, sourcePaneId, targetPaneId, position) => {
      if (position === 'center') {
        get().moveTabToPane(tabId, sourcePaneId, targetPaneId);
        return;
      }
      const { root } = get();
      // Remove tab from source
      let newRoot = updateLeaf(root, sourcePaneId, (leaf) => {
        const newTabIds = leaf.tabIds.filter((t) => t !== tabId);
        return {
          ...leaf,
          tabIds: newTabIds,
          activeTabId:
            leaf.activeTabId === tabId
              ? (newTabIds[0] ?? null)
              : leaf.activeTabId,
        };
      });
      // Collapse empty source
      const src = findNode(newRoot, sourcePaneId);
      if (src && src.type === 'leaf' && src.tabIds.length === 0) {
        const leaves = getAllLeaves(newRoot);
        if (leaves.length > 1) {
          newRoot = removeNode(newRoot, sourcePaneId) ?? newRoot;
        }
      }
      // Split at target
      const target = findNode(newRoot, targetPaneId);
      if (!target || target.type !== 'leaf') {
        // Fallback: create single leaf
        const leaf = createLeaf([tabId], tabId);
        set({ root: leaf, activePaneId: leaf.id });
        return;
      }

      const direction: SplitDirection =
        position === 'left' || position === 'right' ? 'horizontal' : 'vertical';
      const isLeading = position === 'left' || position === 'top';
      const newLeaf = createLeaf([tabId], tabId);

      // Same-direction collapse into parent
      const parentSplit = findParentSplit(newRoot, targetPaneId);
      if (parentSplit && parentSplit.direction === direction) {
        const idx = parentSplit.children.findIndex((c) => c.id === targetPaneId);
        const insertIdx = isLeading ? idx : idx + 1;
        const newChildren = [...parentSplit.children];
        newChildren.splice(insertIdx, 0, newLeaf);
        const count = newChildren.length;
        const scaleFactor = (count - 1) / count;
        const newSizes = parentSplit.sizes.map((s) => s * scaleFactor);
        newSizes.splice(insertIdx, 0, 100 / count);
        const updated: SplitNode = {
          ...parentSplit,
          children: newChildren,
          sizes: newSizes,
        };
        newRoot = replaceNode(newRoot, parentSplit.id, updated);
      } else {
        // Create nested split
        const split: SplitNode = {
          type: 'split',
          id: genId(),
          direction,
          children: isLeading ? [newLeaf, target] : [target, newLeaf],
          sizes: [50, 50],
        };
        newRoot = replaceNode(newRoot, targetPaneId, split);
      }

      set({ root: newRoot, activePaneId: newLeaf.id, fullscreenPaneId: null });
    },

    splitPane: (targetPaneId, direction, position, tabId) => {
      const { root } = get();
      // Find and remove tab from source if it exists in a pane
      const srcLeaf = findLeafContainingTab(root, tabId);
      let newRoot = root;
      if (srcLeaf) {
        newRoot = updateLeaf(root, srcLeaf.id, (leaf) => {
          const newTabIds = leaf.tabIds.filter((t) => t !== tabId);
          return {
            ...leaf,
            tabIds: newTabIds,
            activeTabId:
              leaf.activeTabId === tabId
                ? (newTabIds[0] ?? null)
                : leaf.activeTabId,
          };
        });
        // Collapse empty source
        const updated = findNode(newRoot, srcLeaf.id);
        if (updated && updated.type === 'leaf' && updated.tabIds.length === 0) {
          const leaves = getAllLeaves(newRoot);
          if (leaves.length > 1) {
            newRoot = removeNode(newRoot, srcLeaf.id) ?? newRoot;
          }
        }
      }

      const target = findNode(newRoot, targetPaneId);
      if (!target || target.type !== 'leaf') return;

      const isLeading = position === 'before';
      const newLeaf = createLeaf([tabId], tabId);

      const parentSplit = findParentSplit(newRoot, targetPaneId);
      if (parentSplit && parentSplit.direction === direction) {
        const idx = parentSplit.children.findIndex((c) => c.id === targetPaneId);
        const insertIdx = isLeading ? idx : idx + 1;
        const newChildren = [...parentSplit.children];
        newChildren.splice(insertIdx, 0, newLeaf);
        const count = newChildren.length;
        const scaleFactor = (count - 1) / count;
        const newSizes = parentSplit.sizes.map((s) => s * scaleFactor);
        newSizes.splice(insertIdx, 0, 100 / count);
        const updated: SplitNode = {
          ...parentSplit,
          children: newChildren,
          sizes: newSizes,
        };
        newRoot = replaceNode(newRoot, parentSplit.id, updated);
      } else {
        const split: SplitNode = {
          type: 'split',
          id: genId(),
          direction,
          children: isLeading ? [newLeaf, target] : [target, newLeaf],
          sizes: [50, 50],
        };
        newRoot = replaceNode(newRoot, targetPaneId, split);
      }

      set({ root: newRoot, activePaneId: newLeaf.id, fullscreenPaneId: null });
    },

    closeTabInPane: (paneId, tabId) => {
      const { root } = get();
      let newRoot = updateLeaf(root, paneId, (leaf) => {
        const idx = leaf.tabIds.indexOf(tabId);
        const newTabIds = leaf.tabIds.filter((t) => t !== tabId);
        let newActive = leaf.activeTabId;
        if (leaf.activeTabId === tabId) {
          newActive =
            newTabIds.length > 0
              ? (newTabIds[Math.min(idx, newTabIds.length - 1)] ?? null)
              : null;
        }
        return { ...leaf, tabIds: newTabIds, activeTabId: newActive };
      });

      // Collapse empty pane if not the only one
      const pane = findNode(newRoot, paneId);
      if (pane && pane.type === 'leaf' && pane.tabIds.length === 0) {
        const leaves = getAllLeaves(newRoot);
        if (leaves.length > 1) {
          newRoot = removeNode(newRoot, paneId) ?? newRoot;
          const remainingLeaves = getAllLeaves(newRoot);
          set({
            root: newRoot,
            activePaneId: remainingLeaves[0]?.id ?? get().activePaneId,
          });
          return;
        }
      }
      set({ root: newRoot });
    },

    collapsePane: (paneId) => {
      const { root } = get();
      const leaves = getAllLeaves(root);
      if (leaves.length <= 1) return;
      const newRoot = removeNode(root, paneId) ?? root;
      const remaining = getAllLeaves(newRoot);
      set({
        root: newRoot,
        activePaneId: remaining[0]?.id ?? get().activePaneId,
        fullscreenPaneId: null,
      });
    },

    toggleFullscreen: (paneId) => {
      set((s) => ({
        fullscreenPaneId: s.fullscreenPaneId === paneId ? null : paneId,
      }));
    },

    updateSizes: (splitId, sizes) => {
      set((s) => {
        const node = findNode(s.root, splitId);
        if (!node || node.type !== 'split') return s;
        return { root: replaceNode(s.root, splitId, { ...node, sizes }) };
      });
    },

    getPaneForTab: (tabId) => {
      const leaf = findLeafContainingTab(get().root, tabId);
      return leaf?.id ?? null;
    },
  };
});
