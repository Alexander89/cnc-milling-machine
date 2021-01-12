import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'
import { BehaviorSubject, Observable, Subject } from 'rxjs'

export const programInfoC = t.type({
  name: t.string,
  path: t.string,
  size: t.number,
  linesOfCode: t.number,
  createDateTs: t.number,
  modifiedDateTs: t.number
})
export type ProgramInfo = t.TypeOf<typeof programInfoC>

export const availableProgramsMessageC = t.type({
  type: t.literal('WsAvailableProgramsMessage'),
  progs: t.array(programInfoC),
  inputDir: t.array(t.string)
})
export type AvailableProgramsMessage = t.TypeOf<typeof availableProgramsMessageC>

export const isAvailableProgramsMessage = (msg: object): msg is AvailableProgramsMessage =>
  isRight(availableProgramsMessageC.decode(msg))

export const loadProgramC = t.type({
  type: t.literal('loadProgram'),
  programName: t.string,
  program: t.string,
  invertZ: t.boolean,
  scale: t.number
})
export type LoadProgram = t.TypeOf<typeof loadProgramC>

export const isLoadProgramMessage = (msg: object): msg is LoadProgram =>
  isRight(loadProgramC.decode(msg))

export const saveProgramC = t.type({
  type: t.literal('saveProgram'),
  programName: t.string,
  ok: t.boolean
})
export type SaveProgram = t.TypeOf<typeof saveProgramC>

export const isSaveProgramMessage = (msg: object): msg is SaveProgram =>
  isRight(saveProgramC.decode(msg))

export const deleteProgramC = t.type({
  type: t.literal('deleteProgram'),
  programName: t.string,
  ok: t.boolean
})
export type DeleteProgram = t.TypeOf<typeof deleteProgramC>

export const isDeleteProgramMessage = (msg: object): msg is DeleteProgram =>
  isRight(deleteProgramC.decode(msg))

export const startProgramC = t.type({
  type: t.literal('startProgram'),
  programName: t.string
})
export type StartProgram = t.TypeOf<typeof startProgramC>

export const isStartProgramMessage = (msg: object): msg is StartProgram =>
  isRight(startProgramC.decode(msg))

export const cancelProgramC = t.type({
  type: t.literal('cancelProgram'),
  ok: t.boolean
})
export type CancelProgram = t.TypeOf<typeof cancelProgramC>

export const isCancelProgramMessage = (msg: object): msg is CancelProgram =>
  isRight(cancelProgramC.decode(msg))

export const programReplyC = t.type({
  type: t.literal('reply'),
  msg: t.union([availableProgramsMessageC, loadProgramC, saveProgramC, deleteProgramC, startProgramC, cancelProgramC]),
  to: t.string
})
export type ProgramReply = t.TypeOf<typeof programReplyC>

export const isProgramReplyMessage = (msg: object): msg is ProgramReply =>
  isRight(programReplyC.decode(msg))

// -------------- Commands

export type GetAvailableProgramsCommand = {
  cmd: 'program'
  action: 'get'
}
export type LoadProgramsCommand = {
  cmd: 'program'
  action: 'load'
  programName: string
}
export type SaveProgramsCommand = {
  cmd: 'program'
  action: 'save'
  programName: string
  program: string
}
export type DeleteProgramsCommand = {
  cmd: 'program'
  action: 'delete'
  programName: string
}
export type StartProgramsCommand = {
  cmd: 'program'
  action: 'start'
  programName: string
  invertZ: boolean
  scale: number
}
export type CancelProgramsCommand = {
  cmd: 'program'
  action: 'cancel'
}

export type ProgramCommand =
  | GetAvailableProgramsCommand
  | LoadProgramsCommand
  | SaveProgramsCommand
  | DeleteProgramsCommand
  | StartProgramsCommand
  | CancelProgramsCommand

// -------------- Services

export type ProgramService = {
  availableProgs$: Observable<AvailableProgramsMessage | undefined>
  loadProg$: Observable<LoadProgram | undefined>
  saveProg$: Observable<SaveProgram>
  deleteProg$: Observable<DeleteProgram>
  startProg$: Observable<StartProgram>
  cancelProg$: Observable<CancelProgram>
}

const programServiceLive = (ws: WebSocket): ProgramService => {
  const availableProgsSub = new BehaviorSubject<AvailableProgramsMessage | undefined>(undefined)
  const loadProgsSub = new BehaviorSubject<LoadProgram | undefined>(undefined)
  const saveProgsSub = new Subject<SaveProgram>()
  const deleteProgsSub = new Subject<DeleteProgram>()
  const startProgsSub = new Subject<StartProgram>()
  const cancelProgsSub = new Subject<CancelProgram>()
  ws.addEventListener('message', ({ data }) => {
    const reply = JSON.parse(data)
    console.log(reply)
    if (isProgramReplyMessage(reply)) {
      const { msg } = reply
      switch (msg.type) {
        case 'WsAvailableProgramsMessage':
          availableProgsSub.next(msg)
          break
        case 'loadProgram':
          loadProgsSub.next(msg)
          break
        case 'saveProgram':
          saveProgsSub.next(msg)
          break
        case 'deleteProgram':
          deleteProgsSub.next(msg)
          break
        case 'startProgram':
          startProgsSub.next(msg)
          break
        case 'cancelProgram':
          cancelProgsSub.next(msg)
          break
      }
    } else if (isAvailableProgramsMessage(reply)) {
      availableProgsSub.next(reply)
    }
  })

  return {
    availableProgs$: availableProgsSub.asObservable(),
    loadProg$: loadProgsSub.asObservable(),
    saveProg$: saveProgsSub.asObservable(),
    deleteProg$: deleteProgsSub.asObservable(),
    startProg$: startProgsSub.asObservable(),
    cancelProg$: cancelProgsSub.asObservable()
  }
}

// const programServiceMock: ControllerService = {
//   availableProgs$: of({ : 'program', x: 0, y: 0, z: 0, freezeX: false, freezeY: false, slow: false })
// }

export const programService = {
  live: programServiceLive
  // mock: programServiceMock
}
