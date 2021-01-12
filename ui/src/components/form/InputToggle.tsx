// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import { Input } from '../Input'
import { ToggleButton } from '../ToggleButton'

type InputToggleProps =
  {
    title: string
    button?: string
  } & (
    {
      type: 'number'
      value: number | undefined
      defaultValue: number
      onChanged: (v: number | undefined) => void
    }
    | {
      type: 'string'
      value: string | undefined
      defaultValue: string
      onChanged: (v: string | undefined) => void
    }
  )

export const InputToggle = (props: InputToggleProps) => {
  const onValueChanged = (v: string) => {
    props.value && props.type === 'number'
      ? props.onChanged(+v)
      : props.type === 'string'
        ? props.onChanged(v)
        : console.log('props.type not supported', props.type)
  }
  const onToggleChanged = (v: boolean) => {
    if (props.type === 'number') {
      props.onChanged(v ? (props.defaultValue || 1) : undefined)
    } else if (props.type === 'string') {
      props.onChanged(v ? (props.defaultValue || '1') : undefined)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between' }}>
      {props.title}
      <div style={{ display: 'flex', flexDirection: 'row-reverse' }}>
        <Input
          value={props.value || props.defaultValue}
          onChanged={onValueChanged}
        />
        <ToggleButton
          value={props.value !== undefined}
          onClick={onToggleChanged}
        >
          {props.button || 'Enable'}
        </ToggleButton>
      </div>
    </div>
  )
}
