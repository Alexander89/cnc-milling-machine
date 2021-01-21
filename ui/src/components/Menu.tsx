// eslint-disable-next-line no-use-before-define
import React from 'react'
import { createUseStyles } from 'react-jss'
import { Mode } from '../types'

type Props = {
  mode: Mode
  onChanged: (mode: Mode) => void
}

export const Menu = ({ mode, onChanged }: Props) => {
  const { main, header, bottoms, active } = useStyle()
  const Entry = ({ m, title }: { m: Mode; title: string }) => (
    <div onClick={() => onChanged(m)} className={mode === m ? active : ''}>
      {title}
    </div>
  )

  return (
    <div className={main}>
      <div className={header}>Rusty-CNC</div>
      <div className={bottoms}>
        <Entry m={'main'} title={'Monitoring'} />
        <Entry m={'program'} title={'Jobs'} />
        <Entry m={'calibrate'} title={'Calibrate'} />
        <Entry m={'settings'} title={'Settings'} />
      </div>
    </div>
  )
}

const useStyle = createUseStyles({
  main: {
    flex: '0 0 180px',
    backgroundColor: '#6d6db0',
    padding: '15px 0px 15px 15px',
    color: 'White'
  },
  header: {
    fontSize: '2em',
    marginBottom: 10
  },
  bottoms: {
    fontSize: '1.2em',
    marginLeft: 15,
    '& div': {
      padding: '15px 0px 15px 10px',
      backgroundColor: '#7777aa',
      marginBottom: 5,
      borderTopLeftRadius: 3,
      borderBottomLeftRadius: 15,
      borderLeft: 'solid 3px #ddd',
      borderBottom: 'solid 1px #999',
      marginRight: 0
    },
    '& div:hover': {
      backgroundColor: '#5e5eb0'
    }
  },
  active: {
    backgroundColor: '#8b8bf1 !important'
  },
  bottomActive: {
    backgroundColor: '#3f3fa4'
  }
})
