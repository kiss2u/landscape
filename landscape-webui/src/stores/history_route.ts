import { defineStore } from "pinia";
import { RouteLocationNormalized } from "vue-router";

export interface HistoryRoute {
  name: string;
  path: string;
  meta: any;
  pinned?: boolean;
}

export const useHistoryRouteStore = defineStore("history_route", {
  state: () => ({
    visitedRoutes: [] as HistoryRoute[],
  }),
  actions: {
    addRoute(route: RouteLocationNormalized) {
      if (!route.path) return;
      if (route.path === "/login") return;

      const existingIndex = this.visitedRoutes.findIndex(
        (r) => r.path === route.path,
      );
      const isExistingPinned =
        existingIndex !== -1 && this.visitedRoutes[existingIndex].pinned;

      const pinned = this.visitedRoutes.filter((r) => r.pinned);
      const unpinned = this.visitedRoutes.filter((r) => !r.pinned);

      if (isExistingPinned) {
        // Visiting a pinned route: keep all pinned and the last unpinned visit
        const recent =
          unpinned.length > 0 ? [unpinned[unpinned.length - 1]] : [];
        this.visitedRoutes = [...pinned, ...recent];

        // Update metadata for the current pinned route
        const current = this.visitedRoutes.find((r) => r.path === route.path);
        if (current) {
          current.name = (route.name as string) || "Home";
          current.meta = route.meta;
        }
      } else {
        // Visiting an unpinned route (new or existing): replace all unpinned ones with this one
        this.visitedRoutes = [
          ...pinned,
          {
            name: (route.name as string) || "Home",
            path: route.path,
            meta: route.meta,
            pinned: false,
          },
        ];
      }
    },
    removeRoute(path: string) {
      const index = this.visitedRoutes.findIndex((r) => r.path === path);
      if (index !== -1) {
        this.visitedRoutes.splice(index, 1);
      }
    },
    togglePin(path: string) {
      const route = this.visitedRoutes.find((r) => r.path === path);
      if (route) {
        route.pinned = !route.pinned;
        // Optional: Move pinned to front? User didn't ask but it's common.
        // For now, let's keep order to minimize confusion unless asked.
        // Actually, if we want pinned items to "stick", usually they are distinct.
        // But the user just said "won't be cleaned".
      }
    },
    clearRoutes() {
      // clear only unpinned? Or all? Usually clear all.
      // But if "pinned won't be cleaned", maybe clearRoutes should keep pinned?
      // "clearRoutes" is not usually called by auto-logic, but by a "Close All" button.
      // Let's make clearRoutes clear all for now, or keep pinned. safely keep pinned.
      this.visitedRoutes = this.visitedRoutes.filter((r) => r.pinned);
    },
  },
  persist: true,
});
