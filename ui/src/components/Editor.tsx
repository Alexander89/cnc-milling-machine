// eslint-disable-next-line no-use-before-define
import * as React from 'react'
import * as monaco from 'monaco-editor'

type Props = {
  style?: React.CSSProperties
  file: string
  diff: string | undefined
  onChanged: (code: string) => void
}

const createEditor = (div: HTMLDivElement, value: string) =>
  monaco.editor.create(div, { value, language: 'gnc' })

const createDiffEditor = (div: HTMLDivElement, original: string, modified: string) => {
  const ed = monaco.editor.createDiffEditor(div)
  ed.setModel({
    original: monaco.editor.createModel(original, 'gnc'),
    modified: monaco.editor.createModel(modified, 'gnc')
  })
  return ed
}

let editor: monaco.editor.IStandaloneCodeEditor | undefined
let editorDif: monaco.editor.IStandaloneDiffEditor | undefined

export const Editor = ({ style, file, diff, onChanged }: Props) => {
  const monacoRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    if (monacoRef?.current) {
      if (diff === undefined) {
        editor = createEditor(monacoRef.current, file)
        editor.onDidBlurEditorText(() => {
          const value = editor?.getModel()?.getValue()
          value && onChanged(value)
        })
      } else {
        editorDif = createDiffEditor(monacoRef.current, file, diff)
        editorDif.getModifiedEditor().onDidBlurEditorText(() => {
          const value = editorDif?.getModel()?.modified.getValue()
          value && onChanged(value)
        })
      }
      return () => {
        editorDif && editorDif.dispose()
        editorDif = undefined
        editor && editor.dispose()
        editor = undefined
      }
    }
    return () => {}
  }, [monacoRef === null, diff === undefined])

  React.useEffect(() => {
    const resizeFn = () => {
      editorDif && editorDif.layout()
      editor && editor.layout()
    }
    window.addEventListener('resize', resizeFn)
    return () => {
      window.removeEventListener('resize', resizeFn)
    }
  }, [])

  React.useEffect(() => {
    if (diff === undefined && editor) {
      const model = editor.getModel()
      model?.setValue(file)
    } else if (editorDif && diff) {
      const model = editorDif.getModel()
      model?.original.setValue(file)
      model?.modified.setValue(diff)
    }
    return () => {}
  }, [file, diff])

  return <div ref={monacoRef} style={{ width: '100%', height: '100%', ...style }}></div>
}
