import { computed, ref, watch } from "vue";
import { defineStore } from "pinia";
import { useVueFlow, type Edge, type Node } from "@vue-flow/core";
import {
  LandscapeFlowEdge,
  LandscapeFlowNode,
  PosotionCalculator,
} from "@/lib/flow";
import { DevStateType, NetDev } from "@/lib/dev";
import { ifaces } from "@/api/network";

export const useTopologyStore = defineStore("topology", () => {
  const devs = ref<NetDev[]>([]);

  const topo_nodes = ref<LandscapeFlowNode[]>([]);
  const topo_edges = ref<LandscapeFlowEdge[]>([]);

  const hide_down_dev = ref(false);

  const nodes_index_map = computed(() => {
    let map = new Map();
    for (const [index, node] of topo_nodes.value.entries()) {
      map.set(node.id, index);
    }
    return map;
  });

  function update_topo(new_value: NetDev[], old_value: NetDev[]) {
    let new_value_f = new_value;

    let { addedNodes, removedNodes } = compare_devs(new_value_f, old_value);
    // console.log(addedNodes);
    // console.log(removedNodes);
    if (addedNodes.length != 0) {
      for (const dev_info of addedNodes) {
        topo_nodes.value.push(
          new LandscapeFlowNode({
            id: `${dev_info.index}`,
            type: "netflow",
            label: dev_info.name,
            position: { x: 0, y: 0 },
            data: dev_info,
          })
        );

        if (dev_info.controller_id != undefined) {
          topo_edges.value.push(
            new LandscapeFlowEdge({
              source: `${dev_info.controller_id}`,
              target: `${dev_info.index}`,
              label: "",
              animated: true,
              // type: 'smoothstep',
              class: undefined,
            })
          );
        }
      }
    }
    if (removedNodes.length != 0) {
      let remove_index = new Set();
      let remove_edge = new Set();
      for (const dev_info of removedNodes) {
        remove_index.add(`${dev_info.index}`);
        remove_edge.add(dev_info.index);
      }
      console.log(remove_index);
      topo_nodes.value = topo_nodes.value.filter(
        (node) => !remove_index.has(node.id)
      );
      topo_edges.value = topo_edges.value.filter(
        (node) =>
          !(remove_edge.has(node.source) || remove_edge.has(node.target))
      );
    }

    let position = new PosotionCalculator();
    if (addedNodes.length != 0 || removedNodes.length != 0) {
      for (const node of topo_nodes.value) {
        position.get_position(node);
      }
    }
  }
  //   watch(devs, async (new_value, old_value) => {
  //     update_topo(new_value, old_value)
  //   });

  async function UPDATE_INFO() {
    let new_devs = await ifaces();
    if (hide_down_dev.value) {
      new_devs = new_devs.filter((node) => {
        return node.dev_status.t !== DevStateType.Down;
      });
    }
    update_topo(new_devs, devs.value);
    devs.value = new_devs;
  }

  function UPDATE_HIDE(value: boolean) {
    hide_down_dev.value = value;
  }

  function FIND_BRIDGE_BY_IFINDEX(ifindex: any): boolean {
    let data = FIND_DEV_BY_IFINDEX(ifindex);
    if (data !== undefined && data.dev_kind === "Bridge") {
      return true;
    }
    return false;
  }

  function FIND_DEV_BY_IFINDEX(ifindex: any): NetDev | undefined {
    for (const dev of devs.value) {
      if (dev.index == ifindex) {
        return dev;
      }
    }
    return undefined;
  }

  return {
    topo_nodes,
    topo_edges,
    hide_down_dev,
    nodes_index_map,
    UPDATE_INFO,
    UPDATE_HIDE,
    FIND_BRIDGE_BY_IFINDEX,
    FIND_DEV_BY_IFINDEX,
  };
});

function compare_devs(
  new_value: NetDev[],
  old_value: NetDev[]
): {
  addedNodes: NetDev[];
  removedNodes: NetDev[];
} {
  let new_nodes = [...new_value];
  let old_nodes = [...old_value];

  const newIds = new Set(new_nodes.map((node) => node.name));
  const oldIds = new Set(old_nodes.map((node) => node.name));

  const addedNodes = new_nodes.filter((node) => !oldIds.has(node.name));
  const removedNodes = old_nodes.filter((node) => !newIds.has(node.name));

  return { addedNodes, removedNodes };
}
