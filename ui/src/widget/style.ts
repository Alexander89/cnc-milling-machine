import { createUseStyles } from 'react-jss'

export const useWidgetStyle = createUseStyles({
  devModeBanner: {
    backgroundColor: 'red',
    color: 'white',
    padding: '5px 10px',
    fontSize: '1.5em',
    textAlign: 'center'
  },
  header: {
    fontSize: '1.5em',
    fontWeight: '900',
    padding: '10px 15px',
    marginBottom: 15,
    backgroundColor: '#6d6db0'
  },
  cardBox: {
    display: 'flex'
  },
  card: {
    margin: '10px 15px',
    borderRadius: 8,
    backgroundColor: '#ddd',
    overflow: 'hidden'
  },
  content: {
    margin: '10px 15px'
  },
  row: {
    display: 'flex',
    marginBottom: 25
  },
  posValue: {
    textAlign: 'center',
    margin: '0px 5px',
    fontSize: '1.4em',
    width: 175,
    '& > div': {
      backgroundColor: 'white',
      borderRadius: 10,
      padding: '15px 5px',
      marginTop: 7,
      fontSize: '1.6em',
      fontWeight: '900'
    }
  },
  cardStretch: {
    display: 'flex',
    margin: '10px 15px',
    backgroundColor: '#ddd',
    overflow: 'hidden',
    borderRadius: 8,
    flexDirection: 'column'
  }
})

/*
  #2E94B9
  #FFFDC0
  #F0B775
  #D25565
*/
