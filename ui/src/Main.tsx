import React, { useEffect, useState } from 'react'
import { render } from 'react-dom'
import { createUseStyles } from 'react-jss'
import { Menu } from './components/Menu'
import { defaultServiceCtx, ServiceCtx } from './services'
import { monitoringService } from './services/monitoring'
import { Monitoring } from './views/Monitoring'

export const Main = () => {
  const { main } = useStyle()
  const [service, setService] = useState(defaultServiceCtx)
  useEffect(() => {
    const ws = new WebSocket("ws://localhost:1506/ws");

    ws.onopen = _ => {
      setService({
        monitoring: monitoringService.live(ws),
      })
    }
  }, [])
  return   <ServiceCtx.Provider value={service} >
    <div className={main}>
      <Menu />
      <Monitoring/>
    </div>
  </ServiceCtx.Provider>
}

const useStyle = createUseStyles({
  main: {
    width: '100vw',
    height: '100vh',
    overflow: 'hidden',
    display: 'flex',
  }
})

render(<Main />, document.getElementById('root'))