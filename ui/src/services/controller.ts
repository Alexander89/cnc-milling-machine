import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'
import { Observable, of, BehaviorSubject } from 'rxjs'

// -------------- Messages

export const ControllerMessageC = t.type({
  type: t.literal('controller'),
  x: t.number,
  y: t.number,
  z: t.number,
  freezeX: t.boolean,
  freezeY: t.boolean,
  slow: t.boolean
})
export type ControllerMessage = t.TypeOf<typeof ControllerMessageC>

export const isControllerMessage = (msg: object): msg is ControllerMessage =>
  isRight(ControllerMessageC.decode(msg))

// -------------- Commands

export type FreezeXCommand = {
  cmd: 'controller'
  action: 'freezeX'
  freeze: boolean
}
export type FreezeYCommand = {
  cmd: 'controller'
  action: 'freezeY'
  freeze: boolean
}
export type SlowControlCommand = {
  cmd: 'controller'
  action: 'slow'
  slow: boolean
}
export type FreezeCommand = FreezeXCommand | FreezeYCommand
export type ControllerCommand = FreezeCommand | SlowControlCommand

// -------------- Services

export type ControllerService = {
  controller$: Observable<ControllerMessage | undefined>
}

const controllerServiceLive = (ws: WebSocket): ControllerService => {
  const controllerSub = new BehaviorSubject<ControllerMessage | undefined>(undefined)
  ws.addEventListener('message', ({ data }) => {
    const msg = JSON.parse(data)
    if (isControllerMessage(msg)) {
      controllerSub.next(msg)
    }
  })

  return {
    controller$: controllerSub.asObservable()
  }
}

const controllerServiceMock: ControllerService = {
  controller$: of({ type: 'controller', x: 0, y: 0, z: 0, freezeX: false, freezeY: false, slow: false })
}

export const controllerService = {
  live: controllerServiceLive,
  mock: controllerServiceMock
}
