// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { Button } from '../components/Button'
import { obs, ServiceCtx, StatusMessage } from '../services'

export const Mode = () => {
  const [status, setStatus] = React.useState<StatusMessage>()
  const service = React.useContext(ServiceCtx)

  obs('status$', setStatus)

  const cancel = () => {
    service?.sendCommand({ cmd: 'program', action: 'cancel' })
  }

  const onOff = (on: boolean) => () => {
    service?.sendCommand({ cmd: 'control', action: 'onOff', on })
  }

  return (
    <div className="card" style={{ minWidth: 560 }}>
      <div className="header">Mode</div>
      <div className="content">
        {status && (
          <>
            <div className="row">
              <div className="modeValue">
                Mode: <div>{status.mode}</div>
              </div>
              <div className="modeValue">
                DevMode: <div>{status.devMode ? 'Yes' : 'No'}</div>
              </div>
            </div>
            <div className="row">
              <div className="modeValue">
                Current Job{' '}
                <div style={{ marginTop: 10 }}>{status.currentProg || '---'}</div>
              </div>
            </div>
            <div className="row" style={{ justifyContent: 'space-around', margin: '12px 0px' }}>
              <div style={{ width: '280px' }}>
                <Button onClick={cancel}>Cancel</Button>
              </div>
              <div style={{ width: '190px' }}>
                <Button onClick={onOff(!status.isSwitchedOn)}>
                  {(status.isSwitchedOn && 'Switch off') || 'Switch on'}
                </Button>
              </div>
            </div>
            <div className="row">
              <div className="modeValue">
                Rotor{' '}
                <div style={{ marginTop: 10 }}>
                  {(status.isSwitchedOn && 'on') || 'off'}
                </div>
              </div>
              <div className="modeValue">
                Todo{' '}
                <div style={{ marginTop: 10 }}>{status.stepsTodo || '---'}</div>
              </div>
              <div className="modeValue">
                Done{' '}
                <div style={{ marginTop: 10 }}>{status.stepsDone || '---'}</div>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  )
}
