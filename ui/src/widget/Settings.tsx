// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { Button } from '../components/Button'
import { InputField, ToggleField } from '../components/form'
import { Input } from '../components/Input'
import { AlertCtx, obs, ServiceCtx } from '../services'
import { Runtime } from '../services/settings'
import { useWidgetStyle } from './style'

export const Settings = () => {
  const [settings, setSettings] = useState<Runtime | undefined>()
  const { cardStretch, header, content } = useWidgetStyle()
  const service = useContext(ServiceCtx)
  const { publish } = useContext(AlertCtx)

  obs('runtime$', (p) => setSettings(p))
  obs('runtimeSaved$', (res) =>
    publish({
      message: res.ok ? 'Settings are applied' : 'Apply settings failed'
    })
  )

  const reload = () =>
    service?.sendCommand({ cmd: 'settings', action: 'getRuntime' })
  const save = () =>
    settings &&
    service?.sendCommand({ cmd: 'settings', action: 'setRuntime', ...settings })

  React.useEffect(() => {
    reload()
  }, [])

  return (
    <div className={cardStretch} style={{ width: 750 }}>
      <div className={header}>Runtime settings</div>
      <div className={content}>
        {(settings && (
          <>
            <div
              style={{
                display: 'flex',
                flexDirection: 'row',
                justifyContent: 'space-between'
              }}
            >
              Input directories (.)
              <div>
                <Input
                  value={settings.inputDir?.join(',') || ''}
                  onChanged={(value) =>
                    setSettings({
                      ...settings,
                      inputDir: value.split(',').map((s) => s.trim())
                    })
                  }
                />
              </div>
            </div>
            <InputField
              type="number"
              title="default speed movement (5 mm/s)"
              value={settings.inputUpdateReduce}
              defaultValue={5}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  defaultSpeed: value
                })
              }
            />
            <InputField
              type="number"
              title="Default rapid speed movement (50 mm/s)"
              value={settings.rapidSpeed}
              defaultValue={50}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  rapidSpeed: value
                })
              }
            />
            <InputField
              type="number"
              title="default object scale (1.0)"
              value={settings.scale}
              defaultValue={1}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  scale: value
                })
              }
            />
            <ToggleField
              title="Invert Z in programs by default"
              value={settings.invertZ}
              defaultValue={false}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  invertZ: value
                })
              }
            />
            <ToggleField
              title="Show system output on console"
              value={settings.showConsoleOutput}
              defaultValue={false}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  showConsoleOutput: value
                })
              }
            />
            <InputField
              type="number"
              title="Input update reduce (10)"
              value={settings.inputUpdateReduce}
              defaultValue={10}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  inputUpdateReduce: value
                })
              }
            />
            <InputField
              type="number"
              title="Console position update reduce (50)"
              value={settings.consolePosUpdateReduce}
              defaultValue={50}
              onChanged={(value) =>
                setSettings({
                  ...settings,
                  consolePosUpdateReduce: value
                })
              }
            />
            <div style={{ display: 'flex', justifyContent: 'space-between' }}>
              <Button onClick={reload}>Reload</Button>
              <Button onClick={save}>Save</Button>
            </div>
          </>
        )) ||
          'loading'}
      </div>
    </div>
  )
}
