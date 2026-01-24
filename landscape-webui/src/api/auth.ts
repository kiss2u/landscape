import { LoginInfo, LoginResult } from "landscape-types/common/auth";
import axiosService from ".";

export async function do_login(login: LoginInfo): Promise<LoginResult> {
  axiosService.defaults.baseURL = "/api/auth";
  let data = await axiosService.post("/login", login);
  axiosService.defaults.baseURL = "/api/src";
  return data.data;
}
