// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'
import { primary, primaryDark, primaryLight } from '../theme'

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
      <div>
        {children}
      </div>
    </div>
  )
}

const useStyle = createUseStyles({
  toggleStyle: {
    '& > div': {
      backgroundColor: `${primary} !important`,
      border: 'solid 1px #d0d0d0',
      borderRadius: 2,
      boxShadow: '2px 2px 2px #555, inset 2px 2px 2px 0px #d6d6e6',
      adjustContent: 'center',
      textAlign: 'center',
      padding: '12px 24px',
      cursor: 'pointer',
      margin: 12
    },
    '& > div:hover': {
      backgroundColor: `${primaryLight} !important`
    },
    '& > div:active': {
      backgroundColor: `${primaryDark} !important`
    }
  },
  toggleStyleActive: {
    '& > div': {
      backgroundColor: `${primaryDark} !important`,
      border: 'solid 1px #c0c0c0',
      boxShadow:
        '2px 2px 2px #555, inset 2px 2px 2px #555, inset -2px -2px 2px 0px #a8a8d6'
    },
    '& > div:hover': {
      backgroundColor: `${primary} !important`
    },
    '& > div:active': {
      backgroundColor: `${primaryDark} !important`
    }
  }
})
