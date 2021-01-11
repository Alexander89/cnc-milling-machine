import { createContext, useContext, useEffect } from 'react'
import { ControllerService, controllerService } from './controller'
import { CncCommand } from './types'
import { broadcastService, BroadcastService } from './broadcast'
import { Observable, OperatorFunction, Subject } from 'rxjs'
import { programService, ProgramService } from './program'

export * from './broadcast'
export * from './controller'
export * from './types'

type Services = BroadcastService & ControllerService & ProgramService
export type Service = {
  sendCommand: (cmd: CncCommand) => void
} & Services

// @ts-ignore
export const mockServiceCtx: Service = {
  sendCommand: (cmd: CncCommand) => {
    console.log(cmd)
  },
  ...broadcastService.mock,
  ...controllerService.mock
}

export const mkServiceCtx = (ws: WebSocket): Service => ({
  sendCommand: (cmd: CncCommand) => {
    ws.send(JSON.stringify(cmd))
  },
  ...broadcastService.live(ws),
  ...controllerService.live(ws),
  ...programService.live(ws)
})

export type AlertMsg = {
  title?: string,
  message: string,
  buttons?: {
    text: string,
    onClick: () => void,
  } []
}

export const ServiceCtx = createContext<Service | undefined>(undefined)

type ObservableType<T> = T extends Observable<infer X> ? X : never

export const obs = <V extends keyof Services, T extends ObservableType<Services[V]> >(srv: V, setter: (value: T) => void, ...pipe: OperatorFunction<any, any>[]) => {
  const service = useContext(ServiceCtx)
  useEffect(() => {
    if (service) {
      // @ts-ignore
      const statusSub = service[srv].pipe(...pipe).subscribe(setter)
      return () => {
        statusSub.unsubscribe()
      }
    }
    return () => undefined
  }, [service])
}

export const mkAlertContext = () => {
  const sub = new Subject<AlertMsg>()
  return {
    alerts$: sub.asObservable(),
    publish: (m: AlertMsg) => sub.next(m)
  }
}
export const alertContext = mkAlertContext()
export const AlertCtx = createContext(alertContext)

export const useAlert = (setter: (msg: AlertMsg) => void) => {
  const alerts = useContext(AlertCtx)
  useEffect(() => {
    const alertSub = alerts.alerts$.subscribe(setter)
    return () => alertSub.unsubscribe()
  }, [])
}
