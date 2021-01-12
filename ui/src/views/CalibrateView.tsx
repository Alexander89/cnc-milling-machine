// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { useViewStyle } from './style'

export const CalibrateView = () => {
  const [status, setStatus] = useState<StatusMessage>()
  const { main, devModeBanner, header, cardBox } = useViewStyle()

  obs('status$', setStatus)

  return <div className={main}>
    {status && status.devMode && <div className={devModeBanner}>running in dev-mode</div>}
    <div className={header} style={{ display: 'block' }}>Calibrate</div>
    <div className={cardBox}>
      <div>
        coming soon
      </div>
    </div>
  </div>
}
