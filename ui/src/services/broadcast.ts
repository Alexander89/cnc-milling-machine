import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'
import { Observable, of, BehaviorSubject, ReplaySubject } from 'rxjs'

// -------------- Messages

export const positionMessageC = t.type({
  type: t.literal('position'),
  x: t.number,
  y: t.number,
  z: t.number
})
export type PositionMessage = t.TypeOf<typeof positionMessageC>

export const isPositionMessage = (msg: object): msg is PositionMessage =>
  isRight(positionMessageC.decode(msg))

export const statusMessageC = t.type({
  type: t.literal('status'),
  mode: t.union([t.literal('manual'), t.literal('program'), t.literal('calibrate')]),
  devMode: t.boolean,
  inOpp: t.boolean,
  currentProg: t.union([t.null, t.string]),
  calibrated: t.boolean,
  stepsTodo: t.number,
  stepsDone: t.number
})
export type StatusMessage = t.TypeOf<typeof statusMessageC>

export const isStatusMessage = (msg: object): msg is StatusMessage =>
  isRight(statusMessageC.decode(msg))

export const infoLvlC = t.union([t.literal('info'), t.literal('warning'), t.literal('error')])
export type InfoLvL = t.TypeOf<typeof infoLvlC>

export const infoMessageC = t.type({
  type: t.literal('info'),
  lvl: infoLvlC,
  message: t.string
})
export type InfoMessage = t.TypeOf<typeof infoMessageC>

export const isInfoMessage = (msg: object): msg is InfoMessage =>
  isRight(infoMessageC.decode(msg))

export type BroadcastMessages = PositionMessage | StatusMessage | InfoMessage

// -------------- Service

export type BroadcastService = {
  position$: Observable<PositionMessage | undefined>
  status$: Observable<StatusMessage | undefined>
  info$: Observable<InfoMessage | undefined>
}
const broadcastServiceLive = (ws: WebSocket): BroadcastService => {
  const posSub = new BehaviorSubject<PositionMessage| undefined>(undefined)
  const statusSub = new BehaviorSubject<StatusMessage| undefined>(undefined)
  const infoSub = new ReplaySubject<InfoMessage>(25)
  ws.addEventListener('message', ({ data }) => {
    const msg = JSON.parse(data)
    if (isPositionMessage(msg)) {
      posSub.next(msg)
    } else if (isStatusMessage(msg)) {
      statusSub.next(msg)
    } else if (isInfoMessage(msg)) {
      infoSub.next(msg)
    }
  })
  return {
    position$: posSub.asObservable(),
    status$: statusSub.asObservable(),
    info$: infoSub.asObservable()
  }
}
const broadcastServiceMock: BroadcastService = {
  position$: of({ type: 'position', x: 10.1, y: -15.6, z: 42.1 }),
  status$: of({
    type: 'status',
    calibrated: false,
    devMode: false,
    inOpp: false,
    mode: 'manual',
    currentProg: null,
    stepsDone: 0,
    stepsTodo: 1
  }),
  info$: of({ type: 'info', lvl: 'warning', message: 'testMessage' })
}

export const broadcastService = {
  live: broadcastServiceLive,
  mock: broadcastServiceMock
}
