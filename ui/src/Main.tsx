// eslint-disable-next-line no-use-before-define
import React, { useEffect, useState } from 'react'
import { render } from 'react-dom'
import { createUseStyles } from 'react-jss'
import { AlertBox } from './components/AlertBox'
import { InfoBar } from './components/InfoBar'
import { Menu } from './components/Menu'
import { alertContext, AlertCtx, mkServiceCtx, Service, ServiceCtx } from './services'
import { Mode } from './types'
import { MainView } from './views/MainView'
import { ProgramView } from './views/ProgramView'
import { SettingsView } from './views/SettingsView'
import { SystemView } from './views/SystemView'

export const Main = () => {
  const { main } = useStyle()
  const [service, setService] = useState<Service>()
  const [mode, setMode] = useState<Mode>('program')
  useEffect(() => {
    const ws = new WebSocket(`ws://${window.location.hostname}:1506/ws`)
    ws.onopen = _ => setService(mkServiceCtx(ws))
  }, [])

  const View = () => {
    switch (mode) {
      case 'main':
        return <MainView />
      case 'program':
        return <ProgramView />
      case 'settings':
        return <SettingsView />
      case 'system':
        return <SystemView />
      default:
        return <></>
    }
  }

  return <ServiceCtx.Provider value={service} >
    <AlertCtx.Provider value={alertContext} >
      <div className={main}>
        <AlertBox />
        <Menu mode={mode} onChanged={setMode}/>
        <div style={{ width: '100%', height: '100vh' }}>
          <View />
          <InfoBar />
        </div>
      </div>
    </AlertCtx.Provider>
  </ServiceCtx.Provider>
}

const useStyle = createUseStyles({
  main: {
    position: 'relative',
    width: '100vw',
    height: '100vh',
    overflow: 'hidden',
    display: 'flex'
  }
})

render(<Main />, document.getElementById('root'))
