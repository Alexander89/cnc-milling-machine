// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { ToggleButton } from '../components/ToggleButton'
import { obs, ServiceCtx, ControllerMessage } from '../services'
import { FreezeCommand } from '../services/controller'

export const Controller = () => {
  const service = useContext(ServiceCtx)
  const [controller, setController] = useState<ControllerMessage>()

  const setFreeze = (action: FreezeCommand['action']) => (freeze: boolean) => {
    service?.sendCommand({ cmd: 'controller', action, freeze })
  }
  const setSlow = (slow: boolean) => {
    service?.sendCommand({ cmd: 'controller', action: 'slow', slow })
  }

  obs('controller$', setController)

  return (
    <div className="card">
      <div className="header" style={{ display: 'flex' }}>
        <span>Controller</span>
        <span style={{ flex: 1 }}></span>{' '}
        <div style={{ width: 250, display: 'inline-block', margin: '-20px' }}>
          {controller && (
            <ToggleButton value={controller.slow} onClick={setSlow}>
              Move Slow
            </ToggleButton>
          )}
        </div>
      </div>
      <div className="content">
        {controller && (
          <div className="row">
            <div className="posValue">
              X<div>{(controller.x * 100).toFixed(controller.slow ? 1 : 0)}%</div>
              <ToggleButton
                value={controller.freezeX}
                onClick={setFreeze('freezeX')}
              >
                Freeze
              </ToggleButton>
            </div>
            <div className="posValue">
              Y<div>{(controller.y * 100).toFixed(controller.slow ? 1 : 0)}%</div>
              <ToggleButton
                value={controller.freezeY}
                onClick={setFreeze('freezeY')}
              >
                Freeze
              </ToggleButton>
            </div>
            <div className="posValue">
              Z<div>{(controller.z * 100).toFixed(controller.slow ? 1 : 0)}%</div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
