// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { Button } from '../components/Button'
import { InputField, InputToggle, ToggleField } from '../components/form'
import { AlertCtx, obs, ServiceCtx } from '../services'
import { System as SystemSettings, MotorSettings } from '../services/settings'
import { useWidgetStyle } from './style'

export const System = () => {
  const [settings, setSettings] = useState<SystemSettings | undefined>()
  const { cardStretch, header, content } = useWidgetStyle()
  const service = useContext(ServiceCtx)
  const { publish } = useContext(AlertCtx)

  obs('system$', (p) => setSettings(p))
  obs('systemSaved$', (res) =>
    publish({
      message: res.ok
        ? 'Settings are applied. Please restart the System'
        : 'Apply settings failed'
    })
  )

  const reload = () =>
    service?.sendCommand({ cmd: 'settings', action: 'getSystem' })
  const save = () =>
    settings &&
    service?.sendCommand({ cmd: 'settings', action: 'setSystem', ...settings })

  React.useEffect(() => {
    reload()
  }, [])

  return (
    <div className={cardStretch} style={{ width: 1600 }}>
      <div className={header}>
        System Settings{' '}
        <span style={{ color: 'red', fontSize: '20' }}>(requires restart)</span>
      </div>
      <div className={content}>
        {(settings && (
          <>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <MotorBox
                title="Motor X"
                motor={settings.motorX}
                defaultSettings={{
                  maxStepSpeed: 200,
                  pull: 18,
                  dir: 27,
                  invertDir: false,
                  ena: 1,
                  minStop: 21,
                  maxStop: 20,
                  stepSize: 0.004
                }}
                onChange={(m) =>
                  setSettings({
                    ...settings,
                    motorX: m
                  })
                }
              />
              <MotorBox
                title="Motor Y"
                motor={settings.motorY}
                defaultSettings={{
                  maxStepSpeed: 200,
                  pull: 22,
                  dir: 23,
                  invertDir: false,
                  ena: 1,
                  minStop: 19,
                  maxStop: 26,
                  stepSize: 0.004
                }}
                onChange={(m) =>
                  setSettings({
                    ...settings,
                    motorY: m
                  })
                }
              />
              <MotorBox
                title="Motor Z"
                motor={settings.motorZ}
                defaultSettings={{
                  maxStepSpeed: 200,
                  pull: 25,
                  dir: 24,
                  invertDir: false,
                  ena: 1,
                  minStop: 5,
                  maxStop: 6,
                  stepSize: 0.004
                }}
                onChange={(m) =>
                  setSettings({
                    ...settings,
                    motorZ: m
                  })
                }
              />
            </div>
            <div style={{ marginTop: 25, width: 550 }}>
              <InputToggle
                title="Calibrate Z Gpio Pin"
                type="number"
                value={settings.calibrateZGpio}
                defaultValue={16}
                onChanged={(value: number | undefined) =>
                  setSettings({
                    ...settings,
                    calibrateZGpio: value
                  })
                }
              />
              on_off_gpio
              <InputToggle
                title="On/Off switch Gpio Pin"
                type="number"
                value={settings.onOffGpio}
                defaultValue={12}
                onChanged={(value: number | undefined) =>
                  setSettings({
                    ...settings,
                    onOffGpio: value
                  })
                }
              />
              <InputField
                type="number"
                title="Delay after switch on the actor [sec]"
                value={settings.switchOnOffDelay}
                defaultValue={3}
                onChanged={(value) =>
                  setSettings({
                    ...settings,
                    switchOnOffDelay: value
                  })
                }
              />
              <ToggleField
                title="Developer mode"
                defaultValue={false}
                value={settings.devMode}
                onChanged={(value) =>
                  setSettings({
                    ...settings,
                    devMode: value
                  })
                }
              />
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <Button onClick={reload}>Reload</Button>
                <Button onClick={save}>Save</Button>
              </div>
            </div>
          </>
        )) ||
          'loading'}
      </div>
    </div>
  )
}

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
        borderRadius: 10
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
        title="Max speed (step/sec)"
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
