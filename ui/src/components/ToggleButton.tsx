// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'

type Props = {
  children: React.ReactNode
  value: boolean
  onClick: (value: boolean) => void
}

export const ToggleButton = ({ children, value, onClick }: Props) => {
  const { toggleStyle, toggleStyleActive } = useStyle()
  const activeClass = value ? [toggleStyle, toggleStyleActive] : [toggleStyle]
  return (
    <div onClick={() => onClick(!value)} className={activeClass.join(' ')}>
      {children}
    </div>
  )
}

const useStyle = createUseStyles({
  toggleStyle: {
    fontSize: '24px !important',
    backgroundColor: '#b0b0d7 !important',
    border: 'solid 1px #d0d0d0',
    borderRadius: 10,
    boxShadow: '3px 3px 3px #555, inset 3px 3px 3px 0px #d6d6e6',
    adjustContent: 'center',
    textAlign: 'center',
    padding: '5px 10px',
    cursor: 'pointer',
    margin: 5
  },
  toggleStyleActive: {
    backgroundColor: '#6d6db0 !important',
    border: 'solid 1px #c0c0c0',
    boxShadow:
      '3px 3px 3px #555, inset 3px 3px 3px #555, inset -3px -3px 3px 0px #a8a8d6'
  }
})
