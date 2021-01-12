// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Settings } from '../widget/Settings'
import { System } from '../widget/System'
import { useViewStyle } from './style'

export const SettingsView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  const { main, devModeBanner, header, cardBox } = useViewStyle()

  obs('status$', setStatus)

  return <div className={main}>
    {status && status.devMode && <div className={devModeBanner}>running in dev-mode</div>}
    <div className={header}>Settings</div>
    <div className={cardBox}>
      <div>
        <Settings />
        <System />
      </div>
    </div>
  </div>
}
