import { NetDev } from "./dev";

export function gen_default_router() {}
export enum FlowNodeType {
  Dev = "netflow",
  Route = "router",
  Client = "client",
}

export type LandscapeNodePosition = { x: number; y: number };
export class LandscapeFlowNode {
  id: string;
  // 渲染的节点类型
  type: string | undefined;
  // 网卡名称
  label: string;
  position: LandscapeNodePosition;
  data: NetDev;

  constructor(obj: {
    id: string;
    type?: string;
    label: string;
    position: LandscapeNodePosition;
    data: NetDev;
  }) {
    this.id = obj.id;
    this.type = obj.type;
    this.label = obj.label;
    this.position = obj.position;
    this.data = obj.data;
  }
}

export class LandscapeFlowEdge {
  // node_id1-node_id2
  id: string;
  // 源
  source: string;
  // 目标
  target: string;
  //
  label: string | undefined;
  // 动画
  animated: boolean;
  type: string | undefined;
  class: string | undefined;

  constructor(obj: {
    source: string;
    target: string;
    type?: string;
    label?: string;
    animated: boolean;
    class?: string;
  }) {
    this.id = `${obj.source}-${obj.target}`;
    this.source = obj.source;
    this.target = obj.target;
    this.label = obj.label;
    this.animated = obj.animated ?? true;
    this.type = obj.type;
    this.class = obj.class;
  }
}

export enum NodePositionType {
  Wan = "wan",
  Route = "router",
  Lan = "lan",
  Other = "other",
  WifiAp = "ap",
  Client = "client",
}

export class PosotionCalculator {
  wan: number;
  lan: number;
  lan_port: number;
  client: number;

  constructor() {
    this.wan = 0;
    this.lan = 0;
    this.lan_port = 0;
    this.client = 0;
  }

  get_position(node: LandscapeFlowNode) {
    switch (node.data.get_topology_type()) {
      case NodePositionType.Wan: {
        node.position.x = 100;
        node.position.y = this.wan;
        this.wan += 140;
        break;
      }
      case NodePositionType.Route: {
        node.position.x = 400;
        node.position.y = 500;
        break;
      }
      case NodePositionType.Lan: {
        node.position.x = 700;
        node.position.y = this.lan;
        this.lan += 120;
        break;
      }
      case NodePositionType.Other: {
        node.position.x = 1000;
        node.position.y = this.lan_port;
        this.lan_port += 120;
        break;
      }
      case NodePositionType.WifiAp: {
        node.position.x = 1000;
        node.position.y = this.lan_port;
        this.lan_port += 120;
        break;
      }
      case NodePositionType.Client: {
        node.position.x = 1300;
        node.position.y = this.client;
        this.client += 100;
        break;
      }
    }
  }
}
