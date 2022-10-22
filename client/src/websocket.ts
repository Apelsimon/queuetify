    
let socket: WebSocket = null;

const connect = (onMessageCb: (ev: MessageEvent<any>) => any) =>  {
    doDisconnect()

    const { location } = window

    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/session/ws`

    console.log('Connecting...')
    socket = new WebSocket(wsUri)

    socket.onopen = () => {
      console.log('Connected')
    }

    socket.onmessage = onMessageCb

    socket.onclose = () => {
      console.log('Disconnected')
      socket = null
    }
}

const doDisconnect = () => {
    if (socket) {
      console.log('Disconnecting...')
      socket.close()
      socket = null
    }
}

const doSend = (msg: string) => {
    if (socket) {
        socket.send(msg)
    }
}

const useWebSocket = (onMessageCb: (ev: MessageEvent<any>) => any) => {
  const doConnect = connect.bind(null, onMessageCb)
  return { doConnect, doDisconnect, doSend }
}

export default useWebSocket