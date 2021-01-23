// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { useState } from 'react'
import { createUseStyles } from 'react-jss'
import { obs } from '../services'
import { ProgramInfo } from '../services/program'

export const ProgramMetaData = () => {
  const [programName, setProgramName] = useState<string>()
  const [programs, setPrograms] = useState<ProgramInfo[]>([])
  const { value } = useStyle()

  obs('loadProg$', (p) => setProgramName(p?.programName))
  obs('deleteProg$', () => setProgramName(undefined))

  obs('availableProgs$', (p) => p && setPrograms(p.progs))

  const Program = ({ prog }: { prog?: ProgramInfo }) => {
    if (!prog) {
      return <></>
    }
    return (
      <div className="content">
        <div className={value}>
          <span>Name:</span> <div>{prog.name}</div>
        </div>
        <div className={value}>
          <span>Size:</span> <div>{(prog.size / 1024).toFixed(1)} Kb</div>
        </div>
        <div className={value}>
          <span>Created:</span>{' '}
          <div>{new Date(prog.createDateTs * 1000).toLocaleString()}</div>
        </div>
        <div className={value}>
          <span>Modified:</span>{' '}
          <div>{new Date(prog.modifiedDateTs * 1000).toLocaleString()}</div>
        </div>
        <div className={value}>
          <span>Lines Of Code:</span> <div>{prog.linesOfCode}</div>
        </div>
      </div>
    )
  }

  return (
    <div className="card" style={{ width: 560 }}>
      <div className="header">Program: {programName || ''}</div>
      {programs && programName && (
        <Program prog={programs.find((p) => p.name === programName)} />
      )}
    </div>
  )
}

const useStyle = createUseStyles({
  value: {
    padding: '7px 15px',
    display: 'flex',
    '& > span': {
      flex: '1'
    },
    '& > div': {
      flex: '2',
      alginText: 'right'
    }
  }
})
