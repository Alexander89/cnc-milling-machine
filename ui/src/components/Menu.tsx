// eslint-disable-next-line no-use-before-define
import React, { useState } from 'react'
import { createUseStyles } from 'react-jss'
import { primary, primaryLight } from '../theme'
import { Mode } from '../types'

type Props = {
  mode: Mode
  onChanged: (mode: Mode) => void
}

export const Menu = ({ mode, onChanged }: Props) => {
  const [visible, setVisible] = useState(false)
  const { main, header, bottoms } = useStyle()

  const Entry = ({ m, title }: {m: Mode; title: string }) => (
    <div onClick={() => onChanged(m)} style={mode === m ? { backgroundColor: `${primary}` } : {}}>
      {title}
    </div>
  )

  return (
    <div className={`${main} mainMenu`}>
      <div className={header} onClick={() => setVisible(!visible)}>Rusty-CNC</div>
      <div className={`${bottoms} hideMobile`}>
        <Entry m={'main'} title={'Monitoring'} />
        <Entry m={'program'} title={'Jobs'} />
        <Entry m={'calibrate'} title={'Calibrate'} />
        <Entry m={'settings'} title={'Settings'} />
      </div>
      {visible && (
        <div className={`${bottoms} showMobile`}>
          <Entry m={'main'} title={'Monitoring'} />
          <Entry m={'program'} title={'Jobs'} />
          <Entry m={'calibrate'} title={'Calibrate'} />
          <Entry m={'settings'} title={'Settings'} />
        </div>
      )}
    </div>
  )
}

const useStyle = createUseStyles({
  main: {
    flex: '0 0 180px',
    backgroundColor: primary,
    padding: '15px 0px 15px 15px'
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
      backgroundColor: primaryLight,
      marginBottom: 5,
      borderTopLeftRadius: 3,
      borderBottomLeftRadius: 15,
      borderLeft: 'solid 3px #ddd',
      borderBottom: 'solid 1px #999',
      marginRight: 0,
      cursor: 'pointer'
    },
    '& div:hover': {
      backgroundColor: primaryLight
    }
  },
  bottomActive: {
    backgroundColor: '#3f3fa4'
  }
})
