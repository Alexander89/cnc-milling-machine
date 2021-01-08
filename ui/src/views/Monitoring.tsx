// eslint-disable-next-line no-use-before-define
import React, { useContext, useEffect, useState } from 'react'
import { createUseStyles } from 'react-jss'
import { ToggleButton } from '../components/ToggleButton'
import { ServiceCtx } from '../services'
import { FreezeCommand } from '../services/controller'
import { PositionMessage, ControllerMessage, StatusMessage } from '../services/types'

export const Monitoring = () => {
  const [pos, setPos] = useState<PositionMessage>()
  const [controller, setController] = useState<ControllerMessage>()
  const [status, setStatus] = useState<StatusMessage>()
  const service = useContext(ServiceCtx)
  const { main, devModeBanner, header, cardBox, card, row, posValue } = useStyle()

  const setFreeze = (action: FreezeCommand['action']) => (freeze: boolean) => {
    service.controller.sendCommand({ cmd: 'controller', action, freeze })
  }
  const setSlow = (slow: boolean) => {
    service.controller.sendCommand({ cmd: 'controller', action: 'slow', slow })
  }

  useEffect(() => {
    if (service) {
      const posSub = service.monitoring.position$.subscribe(setPos)
      const statusSub = service.monitoring.status$.subscribe(setStatus)
      const controllerSub = service.controller.controller$.subscribe(setController)
      return () => {
        posSub.unsubscribe()
        controllerSub.unsubscribe()
        statusSub.unsubscribe()
      }
    }
    return () => undefined
  }, [service])

  return <div className={main}>
    {status && status.devMode && <div className={devModeBanner}>running in dev-mode</div>}
    <div className={header}>Monitoring</div>
    <div className={cardBox}>
      <div className={card}>
        <div className={header}>Position</div>
        {pos && (
          <div className={row}>
            <div className={posValue}>X: <div>{(pos.x / 10).toFixed(2)} cm</div></div>
            <div className={posValue}>Y: <div>{(pos.y / 10).toFixed(2)} cm</div></div>
            <div className={posValue}>Z: <div>{(pos.z / 10).toFixed(2)} cm</div></div>
          </div>
        )}
      </div>
     <div className={card}>
        <div className={header}>Controller {controller && <ToggleButton value={controller.slow} onClick={setSlow}>slow</ToggleButton>}</div>
        {controller && (
          <div className={row}>
            <div className={posValue}>X: <div>{(controller.x * 100).toFixed(controller.slow ? 1 : 0)}%</div><ToggleButton value={controller.freezeX} onClick={setFreeze('freezeX')}>Freeze</ToggleButton></div>
            <div className={posValue}>Y: <div>{(controller.y * 100).toFixed(controller.slow ? 1 : 0)}%</div><ToggleButton value={controller.freezeY} onClick={setFreeze('freezeY')}>Freeze</ToggleButton></div>
            <div className={posValue}>Z: <div>{(controller.z * 100).toFixed(controller.slow ? 1 : 0)}%</div></div>
          </div>
        )}
      </div>
      <div className={card}>
        <div className={header}>Mode</div>
        {status && (
          <div className={row}>
            <div className={posValue}>Mode: <div>{status.mode}</div></div>
            <div className={posValue}>DevMode: <div>{status.devMode ? 'Yes' : 'No'}</div></div>
          </div>
        )}
      </div>
      {status && (
        <>
          <div>
            <div>Current Job</div>
            <div>{status.currentProg}</div>
          </div>
          <div>
            <div>Mode</div>
            <div>{status.mode}</div>
            {status.devMode && <div>in devMode</div>}
          </div>
        </>
      )}
    </div>
  </div>
}

const useStyle = createUseStyles({
  main: {
    flex: '1'
  },
  devModeBanner: {
    backgroundColor: 'red',
    color: 'white',
    padding: '5px 10px',
    fontSize: '1.5em',
    textAlign: 'center'
  },
  header: {
    fontSize: '1.5em',
    fontWeight: '900',
    padding: '5px 10px',
    marginBottom: 15
  },
  cardBox: {
    display: 'flex'
  },
  card: {
    border: 'solid 2px #808080',
    margin: '5px 5px',
    padding: '5px 10px',
    borderRadius: 8,
    backgroundColor: '#ddd'
  },
  row: {
    display: 'flex'
  },
  posValue: {
    textAlign: 'center',
    padding: '5px 10px',
    fontSize: '1.4em',
    width: 150,
    borderRight: '',
    '& > div': {
      fontSize: '1.6em',
      fontWeight: '900'
    }
  }
})
