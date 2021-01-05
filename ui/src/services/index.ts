import { createContext } from "react";
import { MonitoringService, monitoringService } from "./monitoring"



export type Service = {
  monitoring: MonitoringService
}
export const defaultServiceCtx: Service = {
  monitoring: monitoringService.mock
}
export const ServiceCtx = createContext(defaultServiceCtx)