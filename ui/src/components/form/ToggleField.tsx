// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { ToggleButton } from '../ToggleButton'

type ToggleFieldProps =
{
  title: string
  button?: string
  value: boolean | undefined
  defaultValue: boolean
  onChanged: (v: boolean) => void
}

export const ToggleField = (props: ToggleFieldProps) =>
  <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between' }}>
    {props.title}
    <div>
      <ToggleButton
        value={props.value || props.defaultValue}
        onClick={props.onChanged}>
          {props.button || 'Enable'}
      </ToggleButton>
    </div>
  </div>
