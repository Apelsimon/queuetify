
let socket: WebSocket = null;

const connect = () =>  {
    disconnect()

    const { location } = window

    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/ws`

    console.log('Connecting...')
    socket = new WebSocket(wsUri)

    socket.onopen = () => {
      console.log('Connected')
    }

    socket.onmessage = (ev) => {
      console.log('Received: ' + ev.data)
    }

    socket.onclose = () => {
      console.log('Disconnected')
      socket = null
    }
}

const disconnect = () => {
    if (socket) {
      console.log('Disconnecting...')
      socket.close()
      socket = null
    }
}

const send = (msg: string) => {
    if (socket) {
        socket.send(msg)
    }
}

const useWebSocket = () => {
    return { connect, disconnect, send }
}

export default useWebSocket