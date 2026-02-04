import { ifaces } from "@/api/network";
import { DevStateType, NetDev } from "@/lib/dev";
import { ZoneType } from "@/lib/service_ipconfig";
import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";

export const useIfaceNodeStore = defineStore(
  "iface_node",
  () => {
    const net_devs = ref<NetDev[]>([]);

    const hide_down_dev = ref(false);
    const view_locked = ref(true);

    const nodes = ref<any>([]);
    const edges = ref<any>([]);

    const bridges = ref<any>([]);
    const eths = ref<any>([]);

    const node_call_back = ref<any>();

    watch(
      [net_devs, hide_down_dev],
      async ([new_value, is_hiding], _old_value) => {
        const tmp_nodes: any[] = [];
        const tmp_edges: any[] = [];
        const new_bridges: any[] = [];
        const new_eths: any[] = [];
        let wan_y = 0;
        let right_y = 0;
        let left_y = 0;
        for (const each of new_value) {
          if (each.dev_type === "Loopback") {
            continue;
          }

          if (is_hiding) {
            if (each.dev_status.t === DevStateType.Down) {
              continue;
            }
          }

          if (each.dev_kind === "bridge") {
            new_bridges.push({
              label: each.name,
              value: each.name,
              ifindex: each.index,
            });
          } else {
            if (each.zone_type !== ZoneType.Wan) {
              new_eths.push({
                label: each.name,
                value: each.name,
                ifindex: each.index,
              });
            }
          }
          let position = { x: 600, y: 100 };
          if (each.zone_type == ZoneType.Wan) {
            position.x = position.x - 400;
            position.y = position.y + wan_y;
            wan_y += 140;
          } else if (each.controller_id == undefined) {
            position.y = position.y + left_y;
            left_y += 120;
          } else {
            position.x = position.x + 450;
            position.y = position.y + right_y;
            right_y += 120;
          }
          tmp_nodes.push({
            id: `${each.index}`,
            data: each,
            type: "netflow",
            label: each.name,
            position,
          });
          if (each.controller_id != undefined) {
            tmp_edges.push({
              id: `${each.controller_id}-${each.index}`,
              source: `${each.controller_id}`,
              target: `${each.index}`,
              label: "",
              animated: true,
              // type: 'smoothstep',
              class: "normal-edge",
            });
          }
        }
        bridges.value = new_bridges;
        eths.value = new_eths;
        // console.log(nodes.value);

        nodes.value = tmp_nodes;
        edges.value = tmp_edges;

        if (node_call_back.value != undefined && view_locked.value) {
          node_call_back.value();
        }
      },
    );

    async function UPDATE_INFO() {
      net_devs.value = await ifaces();
    }

    async function SETTING_CALL_BACK(call_back: any) {
      node_call_back.value = call_back;
    }

    function FIND_BRIDGE_BY_IFINDEX(ifindex: any): boolean {
      for (const bridge of bridges.value) {
        if (bridge.ifindex == ifindex) {
          return true;
        }
      }
      return false;
    }

    function FIND_DEV_BY_IFINDEX(ifindex: any): NetDev | undefined {
      for (const dev of net_devs.value) {
        if (dev.index == ifindex) {
          return dev;
        }
      }
      return undefined;
    }

    function HIDE_DOWN(value: boolean) {
      hide_down_dev.value = value;
    }

    function TOGGLE_VIEW_LOCK() {
      view_locked.value = !view_locked.value;
    }

    return {
      nodes,
      edges,
      bridges,
      eths,
      hide_down_dev,
      view_locked,
      HIDE_DOWN,
      TOGGLE_VIEW_LOCK,
      UPDATE_INFO,
      SETTING_CALL_BACK,
      FIND_DEV_BY_IFINDEX,
      FIND_BRIDGE_BY_IFINDEX,
    };
  },
  {
    persist: {
      key: "iface_node_v1",
      storage: localStorage,
      pick: ["hide_down_dev", "view_locked"],
    },
  },
);
