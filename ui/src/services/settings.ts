import { isRight } from 'fp-ts/lib/Either'
import * as t from 'io-ts'

export const MotorSettingsC = t.intersection([
  t.type({
    max_step_speed: t.number,
    pull_gpio: t.number,
    dir_gpio: t.number,
    step_size: t.number
  }),
  t.partial({
    ena_gpio: t.number,
    end_left_gpio: t.number,
    end_right_gpio: t.number
  })
], 'MotorSettings')

export const systemC = t.intersection([t.type({
  dev_mode: t.boolean,
  motor_x: MotorSettingsC,
  motor_y: MotorSettingsC,
  motor_z: MotorSettingsC
}),
t.partial({
  calibrate_z_gpio: t.number
})], 'System')
export type System = t.TypeOf<typeof systemC>

export const isSystemMessage = (msg: object): msg is System =>
  isRight(systemC.decode(msg))

export const runtimeC = t.partial({
  input_dir: t.array(t.string),
  input_update_reduce: t.number,
  default_speed: t.number,
  rapid_speed: t.number,
  scale: t.number,
  invert_z: t.boolean,
  show_console_output: t.boolean,
  console_pos_update_reduce: t.number
})
export type Runtime = t.TypeOf<typeof runtimeC>

export const isRuntimeMessage = (msg: object): msg is Runtime =>
  isRight(runtimeC.decode(msg))
