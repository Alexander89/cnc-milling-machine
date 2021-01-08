import { createContext } from 'react'
import { MonitoringService, monitoringService } from './monitoring'
import { ControllerService, controllerService } from './controller'

export type Service = {
  monitoring: MonitoringService
  controller: ControllerService
}
export const defaultServiceCtx: Service = {
  monitoring: monitoringService.mock,
  controller: controllerService.mock
}
export const ServiceCtx = createContext(defaultServiceCtx)
