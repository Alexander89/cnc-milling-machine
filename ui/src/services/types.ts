import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'

export const PositionMessageC = t.type({
  type: t.literal('position'),
  x: t.number,
  y: t.number,
  z: t.number
})
export type PositionMessage = t.TypeOf<typeof PositionMessageC>

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

export const StatusMessageC = t.type({
  type: t.literal('status'),
  mode: t.union([t.literal('manual'), t.literal('program'), t.literal('calibrate')]),
  devMode: t.boolean,
  inOpp: t.boolean,
  currentProg: t.union([t.undefined, t.string]),
  calibrated: t.boolean
})
export type StatusMessage = t.TypeOf<typeof StatusMessageC>

export type Messages = PositionMessage | StatusMessage

export const isPositionMessage = (msg: object): msg is PositionMessage =>
  isRight(PositionMessageC.decode(msg))

export const isControllerMessage = (msg: object): msg is ControllerMessage =>
  isRight(ControllerMessageC.decode(msg))

export const isStatusMessage = (msg: object): msg is StatusMessage =>
  isRight(StatusMessageC.decode(msg))
