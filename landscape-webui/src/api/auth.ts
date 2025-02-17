import { LoginInfo, LoginResult } from "@/lib/auth";
import api from ".";

export async function do_login(login: LoginInfo): Promise<LoginResult> {
  let data = await api.auth.post("login", login);
  return new LoginResult(data.data);
}
