// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Settings } from '../widget/Settings'
import { System } from '../widget/System'

export const SettingsView = () => {
  const [status, setStatus] = useState<StatusMessage>()

  obs('status$', setStatus)

  return (
    <div className="viewMain">
      {status && status.devMode && (
        <div className="viewDevModeBanner">running in dev-mode</div>
      )}
      <div className="viewHeader">Settings</div>
      <div className="viewCardBox">
        <div>
          <Settings />
          <System />
        </div>
      </div>
    </div>
  )
}
