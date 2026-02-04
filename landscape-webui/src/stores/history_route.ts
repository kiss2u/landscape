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

      const index = this.visitedRoutes.findIndex((r) => r.path === route.path);
      if (index !== -1) {
        // Update existing route metadata and name (important for i18n updates)
        const existing = this.visitedRoutes[index];
        existing.name = (route.name as string) || "Home";
        existing.meta = route.meta;
      } else {
        // Limit check
        if (this.visitedRoutes.length >= 10) {
          // Find first unpinned route to remove
          const unpinnedIndex = this.visitedRoutes.findIndex((r) => !r.pinned);
          if (unpinnedIndex !== -1) {
            this.visitedRoutes.splice(unpinnedIndex, 1);
          } else {
            // All are pinned, do not add new route (or replace last one? user said "don't increase")
            // If all 10 are pinned, we can't add a new one.
            return;
          }
        }

        this.visitedRoutes.push({
          name: (route.name as string) || "Home",
          path: route.path,
          meta: route.meta,
          pinned: false,
        });
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
