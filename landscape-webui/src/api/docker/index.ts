import { ServiceStatus } from "@/lib/services";
import { DockerContainerSummary, DockerImageSummary } from "@/lib/docker";
import axiosService from "@/api";
import { DockerCmd, PullImgTask } from "@/rust_bindings/common/docker";

export async function get_docker_status(): Promise<ServiceStatus> {
  let data = await axiosService.get("sys_service/docker/status");
  //   console.log(data.data);
  return new ServiceStatus(data.data);
}

export async function start_docker_service(): Promise<ServiceStatus> {
  let data = await axiosService.post("sys_service/docker/status");
  //   console.log(data.data);
  return new ServiceStatus(data.data);
}
export async function stop_docker_service(): Promise<ServiceStatus> {
  let data = await axiosService.delete("sys_service/docker/status");
  //   console.log(data.data);
  return new ServiceStatus(data.data);
}

export async function get_docker_container_summarys(): Promise<
  DockerContainerSummary[]
> {
  let data = await axiosService.get("sys_service/docker/container_summarys");
  // console.log(data.data);
  return data.data.map((d: any) => new DockerContainerSummary(d));
}

export async function start_container(name: string): Promise<any> {
  let data = await axiosService.post(`sys_service/docker/start/${name}`);
  console.log(data.data);
  return;
}

export async function stop_container(name: string): Promise<any> {
  let data = await axiosService.post(
    `sys_service/docker/stop/${name}`,
    undefined,
    {
      timeout: 60000,
    }
  );
  console.log(data.data);
  return;
}

export async function remove_container(name: string): Promise<any> {
  let data = await axiosService.post(`sys_service/docker/remove/${name}`);
  console.log(data.data);
  return;
}

export async function run_cmd(docker_cmd: DockerCmd): Promise<any> {
  let data = await axiosService.post(`sys_service/docker/run_cmd`, docker_cmd);
  console.log(data.data);
  return;
}

export async function get_docker_images(): Promise<DockerImageSummary[]> {
  let data = await axiosService.get("sys_service/docker/images");
  // console.log(data.data);
  return data.data.map((d: any) => new DockerImageSummary(d));
}

export async function pull_docker_image(name: string): Promise<void> {
  await axiosService.post(`sys_service/docker/images/${name}`);
}

export async function get_current_tasks(): Promise<PullImgTask[]> {
  let data = await axiosService.get(`sys_service/docker/images/tasks`);
  return data.data;
}

export async function delete_docker_image(id: string): Promise<void> {
  let data = await axiosService.delete(`sys_service/docker/images/id/${id}`);
  // console.log(data.data);
  return;
}
