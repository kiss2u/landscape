export enum ServiceStatusType {
  Staring = "staring",
  Running = "running",
  Stopping = "stopping",
  Stop = "stop",
}

export class ServiceStatus {
  t: ServiceStatusType;
  message: undefined | string;

  constructor(obj?: { t: ServiceStatusType; message?: string }) {
    this.t = obj?.t ?? ServiceStatusType.Stop;
    this.message = obj?.message;
  }
}
