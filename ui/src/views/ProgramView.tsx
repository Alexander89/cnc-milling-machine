// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { obs, StatusMessage } from '../services'
import { Mode } from '../widget/Mode'
import { ProgramEditor } from '../widget/ProgramEditor'
import { ProgramMetaData } from '../widget/ProgramMetaData'
import { ProgramSelect } from '../widget/ProgramSelect'

export const ProgramView = () => {
  const [status, setStatus] = useState<StatusMessage>()

  obs('status$', setStatus)

  return (
    <div className="viewMain">
      {status && status.devMode && (
        <div className="viewDevModeBanner">running in dev-mode</div>
      )}
      <div className="viewHeader">Program Management</div>
      <div className="viewCardBox ProgramCardBox">
        <ProgramSelect />
        <ProgramEditor />
        <div className="ProgViewInfo">
          <Mode />
          <ProgramMetaData />
        </div>
      </div>
    </div>
  )
}
