// eslint-disable-next-line no-use-before-define
import React, { useEffect, useState } from 'react'
import { render } from 'react-dom'
import { AlertBox } from './components/AlertBox'
import { InfoBar } from './components/InfoBar'
import { Menu } from './components/Menu'
import {
  alertContext,
  AlertCtx,
  mkServiceCtx,
  Service,
  ServiceCtx
} from './services'
import { Mode } from './types'
import { MainView } from './views/MainView'
import { ProgramView } from './views/ProgramView'
import { SettingsView } from './views/SettingsView'
import { CalibrateView } from './views/CalibrateView'
import './responsive.css'

export const Main = () => {
  const [service, setService] = useState<Service>()
  const [mode, setMode] = useState<Mode>('program')

  const connect = () => {
    const ws = new WebSocket(`ws://${window.location.hostname}:1506/ws`)
    ws.onopen = (_) => setService(mkServiceCtx(ws))
    ws.addEventListener('close', () => setTimeout(connect, 1000))
  }
  useEffect(() => connect(), [])

  const View = () => {
    switch (mode) {
      case 'main':
        return <MainView />
      case 'program':
        return <ProgramView />
      case 'settings':
        return <SettingsView />
      case 'calibrate':
        return <CalibrateView />
      default:
        return <></>
    }
  }

  return (
    <ServiceCtx.Provider value={service}>
      <AlertCtx.Provider value={alertContext}>
        <div className="main">
          <AlertBox />
          <Menu mode={mode} onChanged={setMode} />
          <div style={{ width: '100%', height: '100vh' }}>
            <View />
            <InfoBar />
          </div>
        </div>
      </AlertCtx.Provider>
    </ServiceCtx.Provider>
  )
}

render(<Main />, document.getElementById('root'))
