// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'

export const CalibrateView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  obs('status$', setStatus)

  return (
    <div className="viewMain">
      {status && status.devMode && (
        <div className="viewDevModeBanner">running in dev-mode</div>
      )}
      <div className="viewHeader" style={{ display: 'block' }}>
        Calibrate
      </div>
      <div className="viewCardBox">
        <div>coming soon</div>
      </div>
    </div>
  )
}
