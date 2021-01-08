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
  return <div onClick={() => onClick(!value)} className={activeClass.join(' ')}>
    {children}
  </div>
}

const useStyle = createUseStyles({
  toggleStyle: {
    fontSize: 14,
    backgroundColor: '#b0b0d7',
    border: 'solid 1px #d0d0d0'
  },
  toggleStyleActive: {
    fontSize: 14,
    backgroundColor: '#6d6db0',
    border: 'solid 1px #c0c0c0'
  }
})
