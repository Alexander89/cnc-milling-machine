import { createUseStyles } from 'react-jss'

export const useViewStyle = createUseStyles({
  main: {
    flex: '1',
    height: 'calc(100vh - 175px)',
    overflow: 'auto'
  },
  devModeBanner: {
    backgroundColor: 'red',
    color: 'white',
    padding: '5px 10px',
    fontSize: '1.5em',
    textAlign: 'center'
  },
  header: {
    fontSize: '1.8em',
    fontWeight: '900',
    padding: '5px 10px',
    paddingTop: 10,
    paddingBottom: 20,
    marginBottom: 10,
    backgroundColor: '#ddd'
  },
  cardBox: {
    display: 'flex'
  },
  content: {

  }
})
