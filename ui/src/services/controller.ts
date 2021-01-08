import { Observable, of, Subject } from 'rxjs'
import * as MSG from './types'

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

export type ControllerService = {
  sendCommand: (cmd: ControllerCommand) => void
  controller$: Observable<MSG.ControllerMessage>
}
const controllerServiceLive = (ws: WebSocket): ControllerService => {
  const controllerSub = new Subject<MSG.ControllerMessage>()
  ws.addEventListener('message', ({ data }) => {
    const msg = JSON.parse(data)
    if (MSG.isControllerMessage(msg)) {
      controllerSub.next(msg)
    }
  })

  const sendCommand = (cmd: ControllerCommand) => {
    ws.send(JSON.stringify(cmd))
  }

  return {
    sendCommand,
    controller$: controllerSub.asObservable()
  }
}
const controllerServiceMock: ControllerService = {
  sendCommand: (_: ControllerCommand) => 0,
  controller$: of({ type: 'controller', x: 0, y: 0, z: 0, freezeX: false, freezeY: false, slow: false })
}

export const controllerService = {
  live: controllerServiceLive,
  mock: controllerServiceMock
}
