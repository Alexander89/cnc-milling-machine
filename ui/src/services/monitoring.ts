import { Observable, of } from "rxjs"

export type Monitoring = {

}

export type Position = {
  x: number
  y: number
  z: number
}

type MonitoringService = {
  position$: Observable<Position>
}
const monitoringServiceLive: MonitoringService = {
  position$: of({x: 10, y: 15, z: 20})
}
const monitoringServiceMock: MonitoringService = {
  position$: of({x: 10, y: 15, z: 20})
}

export const MonitoringService = {
  live: monitoringServiceLive,
  mock: monitoringServiceMock,
}