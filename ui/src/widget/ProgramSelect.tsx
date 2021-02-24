// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useContext, useState } from 'react'
import { createUseStyles } from 'react-jss'
import { Button } from '../components/Button'
import { obs, ServiceCtx } from '../services'
import { AvailableProgramsMessage } from '../services/program'

export const ProgramSelect = () => {
  const [programs, setPrograms] = useState<AvailableProgramsMessage>()
  const [selected, setSelected] = useState<string>()
  const { progList } = useStyle()
  const service = useContext(ServiceCtx)

  obs('availableProgs$', setPrograms)

  React.useEffect(() => {
    refresh()
  }, [])

  const refresh = () => service?.sendCommand({ cmd: 'program', action: 'get' })
  const load = (programName: string) =>
    service?.sendCommand({ cmd: 'program', action: 'load', programName })

  return (
    <div className="cardStretch ProgSelectCardBox">
      <div
        className="header"
        style={{ display: 'flex', justifyContent: 'space-between' }}
      >
        Available Programs
      </div>
      <div
        className="content"
        style={{ display: 'flex', flexDirection: 'column', height: '100%' }}
      >
        <div></div>

        <div
          style={{ display: 'flex', justifyContent: 'space-between' }}
        >
          Search dir: {programs ? programs.inputDir.join(', ') : '---'}
          <div style={{ width: 130, display: 'inline-block' }}>
            <Button onClick={refresh}>Refresh</Button>
          </div>
        </div>

        <div className={progList}>
          {programs
            ? programs.progs.map((p) => (
                <div
                  key={p.name}
                  style={
                    selected === p.name
                      ? { backgroundColor: '#c8c8c8', display: 'flex' }
                      : { cursor: 'pointer', display: 'flex' }
                  }
                  onClick={() => {
                    setSelected(p.name)
                    load(p.name)
                  }}
                >
                  <span style={{ flex: '1' }}>{p.name}</span>{' '}
                  <span>
                    ({new Date(p.modifiedDateTs * 1000).toLocaleDateString()})
                  </span>
                </div>
              ))
            : 'loading'}
        </div>
      </div>
    </div>
  )
}

const useStyle = createUseStyles({
  progList: {
    flex: '1',
    fontSize: '1.4em',
    lineHeight: '1.8em',
    backgroundColor: 'white',
    margin: '10px 0px',
    height: '50vh',
    overflow: 'auto',
    '& > div': {
      padding: 7
    }
  }
})
