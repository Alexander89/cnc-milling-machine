import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'
import { BehaviorSubject, Observable, Subject } from 'rxjs'

export const motorSettingsC = t.intersection([
  t.type({
    maxStepSpeed: t.number,
    pullGpio: t.number,
    dirGpio: t.number,
    invertDir: t.boolean,
    stepSize: t.number,
    acceleration: t.number,
    freeStepSpeed: t.number,
    accelerationDamping: t.number,
    accelerationTimeScale: t.number
  }),
  t.partial({
    enaGpio: t.number,
    endLeftGpio: t.number,
    endRightGpio: t.number
  })
], 'MotorSettings')
export type MotorSettings = t.TypeOf<typeof motorSettingsC>

export const systemC = t.intersection([
  t.type({
    type: t.literal('systemSettings'),
    devMode: t.boolean,
    motorX: motorSettingsC,
    motorY: motorSettingsC,
    motorZ: motorSettingsC,
    switchOnOffDelay: t.number
  }),
  t.partial({
    calibrateZGpio: t.number,
    onOffGpio: t.number
  })
], 'System')
export type System = t.TypeOf<typeof systemC>

export const isSystemMessage = (msg: object): msg is System =>
  isRight(systemC.decode(msg))

export const runtimeC = t.intersection([
  t.type({
    type: t.literal('runtimeSettings')
  }),
  t.partial({
    inputDir: t.array(t.string),
    inputUpdateReduce: t.number,
    defaultSpeed: t.number,
    rapidSpeed: t.number,
    scale: t.number,
    invertZ: t.boolean,
    showConsoleOutput: t.boolean,
    consolePosUpdateReduce: t.number,
    externalInputEnabled: t.boolean
  })
])
export type Runtime = t.TypeOf<typeof runtimeC>

export const isRuntimeMessage = (msg: object): msg is Runtime =>
  isRight(runtimeC.decode(msg))

export const runtimeSavedC = t.type({
  type: t.literal('runtimeSettingsSaved'),
  ok: t.boolean
})
export type RuntimeSaved = t.TypeOf<typeof runtimeSavedC>

export const isRuntimeSavedMessage = (msg: object): msg is RuntimeSaved =>
  isRight(runtimeSavedC.decode(msg))

export const systemSavedC = t.type({
  type: t.literal('systemSettingsSaved'),
  ok: t.boolean
})
export type SystemSaved = t.TypeOf<typeof systemSavedC>

export const isSystemSavedMessage = (msg: object): msg is SystemSaved =>
  isRight(systemSavedC.decode(msg))

export const programReplyC = t.type({
  type: t.literal('reply'),
  to: t.string,
  msg: t.union([systemC, runtimeC, runtimeSavedC, systemSavedC])
})
export type ProgramReply = t.TypeOf<typeof programReplyC>

export const isProgramReplyMessage = (msg: object): msg is ProgramReply =>
  isRight(programReplyC.decode(msg))

// -------------- Commands

export type GetSystemSettingsCommand = {
  cmd: 'settings'
  action: 'getSystem'
}
export type SetSystemSettingsCommand = {
  cmd: 'settings'
  action: 'setSystem'
  devMode: boolean,
  motorX: MotorSettings,
  motorY: MotorSettings,
  motorZ: MotorSettings,
  switchOnOffDelay: number
  calibrateZGpio?: number,
  onOffGpio?: number
}
export type GetRuntimeSettingsCommand = {
  cmd: 'settings'
  action: 'getRuntime'
}
export type SetRuntimeSettingsCommand = {
  cmd: 'settings'
  action: 'setRuntime'
  inputDir?: string[]
  inputUpdateReduce?: number
  defaultSpeed?: number
  rapidSpeed?: number
  scale?: number
  invertZ?: boolean
  showConsoleOutput?: boolean
  consolePosUpdateReduce?: number
  externalInputEnabled?:boolean
}

export type SettingsCommand =
  | GetSystemSettingsCommand
  | SetSystemSettingsCommand
  | GetRuntimeSettingsCommand
  | SetRuntimeSettingsCommand

// -------------- Services

export type SettingsService = {
  system$: Observable<System | undefined>
  systemSaved$: Observable<SystemSaved>
  runtime$: Observable<Runtime | undefined>
  runtimeSaved$: Observable<RuntimeSaved>
}

const settingsServiceLive = (ws: WebSocket): SettingsService => {
  const systemSub = new BehaviorSubject<System | undefined>(undefined)
  const runtimeSavedSub = new Subject<RuntimeSaved>()
  const runtimeSub = new BehaviorSubject<Runtime | undefined>(undefined)
  const systemSavedSub = new Subject<SystemSaved>()
  ws.addEventListener('message', ({ data }) => {
    const reply = JSON.parse(data)
    console.log(reply)
    if (isProgramReplyMessage(reply)) {
      const { msg } = reply
      switch (msg.type) {
        case 'systemSettings':
          systemSub.next(msg)
          break
        case 'runtimeSettingsSaved':
          runtimeSavedSub.next(msg)
          break
        case 'runtimeSettings':
          runtimeSub.next(msg)
          break
        case 'systemSettingsSaved':
          systemSavedSub.next(msg)
          break
      }
    }
  })

  return {
    system$: systemSub.asObservable(),
    systemSaved$: systemSavedSub.asObservable(),
    runtime$: runtimeSub.asObservable(),
    runtimeSaved$: runtimeSavedSub.asObservable()
  }
}

// const settingsServiceMock: ControllerService = {
//   system$: of({ : 'settings', x: 0, y: 0, z: 0, freezeX: false, freezeY: false, slow: false })
// }

export const settingsService = {
  live: settingsServiceLive
  // mock: settingsServiceMock
}
