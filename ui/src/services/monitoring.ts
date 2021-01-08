import { Observable, of, Subject } from 'rxjs'
import * as MSG from './types'

export type MonitoringService = {
  position$: Observable<MSG.PositionMessage>
  status$: Observable<MSG.StatusMessage>
}
const monitoringServiceLive = (ws: WebSocket): MonitoringService => {
  const posSub = new Subject<MSG.PositionMessage>()
  const statusSub = new Subject<MSG.StatusMessage>()
  ws.addEventListener('message', ({ data }) => {
    const msg = JSON.parse(data)
    if (MSG.isPositionMessage(msg)) {
      posSub.next(msg)
    } else if (MSG.isStatusMessage(msg)) {
      statusSub.next(msg)
    }
  })
  return {
    position$: posSub.asObservable(),
    status$: statusSub.asObservable()
  }
}
const monitoringServiceMock: MonitoringService = {
  position$: of({ type: 'position', x: 10.1, y: -15.6, z: 42.1 }),
  status$: of({
    type: 'status',
    calibrated: false,
    devMode: false,
    inOpp: false,
    mode: 'manual',
    currentProg: undefined
  })
}

export const monitoringService = {
  live: monitoringServiceLive,
  mock: monitoringServiceMock
}
