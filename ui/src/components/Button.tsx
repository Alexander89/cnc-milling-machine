// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { createUseStyles } from 'react-jss'
import { primary, primaryDark, primaryLight } from '../theme'

type Props = {
  children: React.ReactNode
  onClick: () => void
}

export const Button = ({ children, onClick }: Props) => {
  const { buttonStyle } = useStyle()
  return (
    <div onClick={() => onClick()} className={buttonStyle}>
      <div>{children}</div>
    </div>
  )
}

const useStyle = createUseStyles({
  buttonStyle: {
    '& > div': {
      backgroundColor: primary,
      border: 'solid 1px #d0d0d0',
      borderRadius: 2,
      boxShadow: '2px 2px 2px #555, inset 2px 2px 2px 0px #d6d6e6',
      adjustContent: 'center',
      textAlign: 'center',
      padding: '12px 24px',
      cursor: 'pointer'
    },
    '& > div:hover': {
      backgroundColor: primaryLight,
      border: 'solid 1px #c0c0c0',
      boxShadow: '2px 2px 2px #555'
    },
    '& > div:active': {
      backgroundColor: primaryDark,
      border: 'solid 1px #c0c0c0',
      boxShadow: '2px 2px 2px #555'
    }
  }
})
