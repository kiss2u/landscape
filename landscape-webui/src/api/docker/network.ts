import axiosService from "@/api";
import { LandscapeDockerNetwork } from "@/lib/docker/network";

export async function get_all_docker_networks(): Promise<
  LandscapeDockerNetwork[]
> {
  let data = await axiosService.get("sys_service/docker/networks");
  return data.data.map((d: any) => new LandscapeDockerNetwork(d));
}
