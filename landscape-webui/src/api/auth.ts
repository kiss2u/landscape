import type { LoginInfo } from "@landscape-router/types/api/schemas";
import { loginHandler } from "@landscape-router/types/api/auth/auth";
import axios from "axios";
import { applyInterceptors } from "@/api";

const authAxios = applyInterceptors(axios.create({ timeout: 30000 }));

export async function do_login(login: LoginInfo) {
  return loginHandler(login);
}

export async function change_password(payload: {
  current_password: string;
  new_password: string;
  confirm_password: string;
}): Promise<void> {
  await authAxios.post("/api/v1/system/config/edit/auth", payload);
}
