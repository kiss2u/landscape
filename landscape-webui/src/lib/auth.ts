export class LoginInfo {
  username: string;
  password: string;

  constructor(obj: any) {
    this.username = obj.username;
    this.password = obj.password;
  }
}

export class LoginResult {
  success: boolean;
  token: string;

  constructor(obj: { success?: boolean; token?: string }) {
    this.success = obj.success ?? false;
    this.token = obj.token ?? "";
  }
}
