import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'

export const PositionMessage = t.type({
  type: t.literal('position'),
  x: t.number,
  y: t.number,
  z: t.number,
})
export type PositionMessage = t.TypeOf<typeof PositionMessage>

export const StatusMessage = t.type({
  type: t.literal('status'),
  mode: t.union([t.literal('manual'), t.literal('program'), t.literal('calibrate')]),
  devMode: t.boolean,
  inOpp: t.boolean,
  currentProg: t.union([t.undefined, t.string]),
  calibrated: t.boolean,
})
export type StatusMessage = t.TypeOf<typeof StatusMessage>

export type Messages = PositionMessage | StatusMessage

export const isPositionMessage = (msg: object): msg is PositionMessage =>
  isRight(PositionMessage.decode(msg))

export const isStatusMessage = (msg: object): msg is StatusMessage =>
  isRight(StatusMessage.decode(msg))
