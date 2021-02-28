// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { InputField, InputToggle, ToggleField } from '../components/form'
import { MotorSettings } from '../services/settings'
import { MotorRampUp } from './MotorRampUp'

type MotorBoxProps = {
  title: string
  motor: MotorSettings
  onChange: (m: MotorSettings) => void
  defaultSettings: {
    pull: number
    dir: number
    invertDir: boolean
    ena: number
    minStop: number
    maxStop: number
    maxStepSpeed: number
    stepSize: number
    acceleration: number
    freeStepSpeed: number
    accelerationDamping: number
    accelerationTimeScale: number
  }
}

export const MotorBox = ({
  title,
  motor,
  onChange,
  defaultSettings
}: MotorBoxProps) => {
  return (
    <div
      style={{
        backgroundColor: '#e5e5e5',
        padding: '15px 10px',
        borderRadius: 10,
        marginBottom: 24
      }}
    >
      <div style={{ fontSize: 24, marginBottom: 25 }}>{title}</div>
      <InputField
        title="Motor Step size (mm)"
        type="number"
        value={motor.stepSize}
        defaultValue={defaultSettings.stepSize}
        onChanged={(value) =>
          onChange({
            ...motor,
            stepSize: value
          })
        }
      />
      <InputField
        title="Free run speed (mm/min)"
        type="number"
        value={motor.freeStepSpeed}
        defaultValue={defaultSettings.freeStepSpeed}
        onChanged={(value) =>
          onChange({
            ...motor,
            freeStepSpeed: value
          })
        }
      />
      <InputField
        title="Max speed (mm/min)"
        type="number"
        value={motor.maxStepSpeed}
        defaultValue={defaultSettings.maxStepSpeed}
        onChanged={(value) =>
          onChange({
            ...motor,
            maxStepSpeed: value
          })
        }
      />
      <InputField
        title="Acceleration"
        type="number"
        value={motor.acceleration}
        defaultValue={defaultSettings.acceleration}
        onChanged={(value) =>
          onChange({
            ...motor,
            acceleration: value
          })
        }
      />
      <InputField
        title="Acceleration damping"
        type="number"
        value={motor.accelerationDamping}
        defaultValue={defaultSettings.accelerationDamping}
        onChanged={(value) =>
          onChange({
            ...motor,
            accelerationDamping: value
          })
        }
      />
      <InputField
        title="Graph time scale"
        type="number"
        value={motor.accelerationTimeScale}
        defaultValue={defaultSettings.accelerationTimeScale}
        onChanged={(value) =>
          onChange({
            ...motor,
            accelerationTimeScale: value
          })
        }
      />
      <MotorRampUp
        stepSize={motor.stepSize}
        freeRunSpeed={motor.freeStepSpeed}
        maxSpeed={motor.maxStepSpeed}
        acceleration={motor.acceleration}
        damping={motor.accelerationDamping}
        time={motor.accelerationTimeScale}
      />
      <InputField
        title="Pull gpio pin number"
        type="number"
        value={motor.pullGpio}
        defaultValue={defaultSettings.pull}
        onChanged={(value) =>
          onChange({
            ...motor,
            pullGpio: value
          })
        }
      />
      <InputField
        title="Direction gpio pin number"
        type="number"
        value={motor.dirGpio}
        defaultValue={defaultSettings.dir}
        onChanged={(value) =>
          onChange({
            ...motor,
            dirGpio: value
          })
        }
      />
      <ToggleField
        title="invert direction"
        defaultValue={motor.invertDir}
        value={motor.invertDir}
        onChanged={(value) =>
          onChange({
            ...motor,
            invertDir: value
          })
        }
      />
      <InputToggle
        title="Enable gpio pin number"
        type="number"
        value={motor.enaGpio}
        defaultValue={defaultSettings.ena}
        onChanged={(value) =>
          onChange({
            ...motor,
            enaGpio: value
          })
        }
      />
      <InputToggle
        title="Min / left end switch"
        type="number"
        value={motor.endLeftGpio}
        defaultValue={defaultSettings.minStop}
        onChanged={(value) =>
          onChange({
            ...motor,
            endLeftGpio: value
          })
        }
      />
      <InputToggle
        title="Max / right end switch"
        type="number"
        value={motor.endRightGpio}
        defaultValue={defaultSettings.maxStop}
        onChanged={(value) =>
          onChange({
            ...motor,
            endRightGpio: value
          })
        }
      />
    </div>
  )
}
