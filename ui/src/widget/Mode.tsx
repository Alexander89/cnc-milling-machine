// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'
import { Button } from '../components/Button'
import { obs, ServiceCtx, StatusMessage } from '../services'
import { useWidgetStyle } from './style'

export const Mode = () => {
  const [status, setStatus] = React.useState<StatusMessage>()
  const service = React.useContext(ServiceCtx)

  const { card, header, row } = useWidgetStyle()
  const { modeValue } = useStyle()
  obs('status$', setStatus)

  const cancel = () => {
    service?.sendCommand({ cmd: 'program', action: 'cancel' })
  }

  return (
    <div className={card}>
      <div className={header}>Mode</div>
      {status && (
        <>
          <div className={row}>
            <div className={modeValue}>Mode: <div>{status.mode}</div></div>
            <div className={modeValue}>DevMode: <div>{status.devMode ? 'Yes' : 'No'}</div></div>
          </div>
          <div className={row}>
            <div className={modeValue}>Current Job <div style={{ marginTop: 10 }}>{status.currentProg || '---'}</div></div>
          </div>
          <div className={row} style={{ justifyContent: 'space-around' }}>
            <div style={{ width: '350px' }}><Button onClick={cancel}>Cancel</Button></div>
          </div>
          <div className={row}>
            <div className={modeValue}>Todo <div style={{ marginTop: 10 }}>{status.stepsTodo || '---'}</div></div>
            <div className={modeValue}>Done <div style={{ marginTop: 10 }}>{status.stepsDone || '---'}</div></div>
          </div>
        </>
      )}
    </div>
  )
}

const useStyle = createUseStyles({
  modeValue: {
    flex: '1',
    textAlign: 'center',
    padding: '5px 10px',
    fontSize: '1.4em',
    width: 220,
    '& > div': {
      backgroundColor: 'white',
      borderRadius: 10,
      padding: '15px 5px',
      marginTop: 7,
      fontSize: '1.6em',
      fontWeight: '900'
    }
  }
})
