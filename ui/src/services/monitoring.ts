import { Observable, of, Subject } from "rxjs"
import { isPositionMessage, isStatusMessage, PositionMessage, StatusMessage } from "./types"

export type MonitoringService = {
  position$: Observable<PositionMessage>
  status$: Observable<StatusMessage>
}
const monitoringServiceLive = (ws: WebSocket): MonitoringService => {
  const posSub = new Subject<PositionMessage>()
  const statusSub = new Subject<StatusMessage>()
  ws.addEventListener('message', ({data}) => {
    const msg = JSON.parse(data)
    if (isPositionMessage(msg)) {
      posSub.next(msg)
    }
    else if (isStatusMessage(msg)) {
      statusSub.next(msg)
    }
  })
  return {
    position$: posSub.asObservable(),
    status$: statusSub.asObservable(),
  }
}
const monitoringServiceMock: MonitoringService = {
  position$: of({type: 'position', x: 10, y: 15, z: 20}),
  status$: of({
    type: 'status',
    calibrated: false,
    devMode: false,
    inOpp: false,
    mode: 'manual',
    currentProg: undefined,
  })
}

export const monitoringService = {
  live: monitoringServiceLive,
  mock: monitoringServiceMock,
}
