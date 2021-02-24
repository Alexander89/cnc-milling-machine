// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { Button } from '../components/Button'
import { InputField, InputToggle, ToggleField } from '../components/form'
import { MotorBox } from '../components/MotorBox'
import { AlertCtx, obs, ServiceCtx } from '../services'
import { System as SystemSettings } from '../services/settings'

export const System = () => {
  const [settings, setSettings] = useState<SystemSettings | undefined>()
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
    <div className="cardStretch SystemSettingsBox">
      <div className="header">
        System Settings{' '}
        <span style={{ color: 'red', fontSize: '20' }}>(requires restart)</span>
      </div>
      <div className="content">
        {(settings && (
          <>
            <div className="MotorSettingsBox">
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
                  stepSize: 0.004,
                  acceleration: 50,
                  freeStepSpeed: 20,
                  accelerationDamping: 0.8,
                  accelerationTimeScale: 2
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
                  stepSize: 0.004,
                  acceleration: 50,
                  freeStepSpeed: 20,
                  accelerationDamping: 0.8,
                  accelerationTimeScale: 2
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
                  stepSize: 0.004,
                  acceleration: 50,
                  freeStepSpeed: 20,
                  accelerationDamping: 0.8,
                  accelerationTimeScale: 2
                }}
                onChange={(m) =>
                  setSettings({
                    ...settings,
                    motorZ: m
                  })
                }
              />
            </div>
            <div className="SystemGeneralSettings">
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
