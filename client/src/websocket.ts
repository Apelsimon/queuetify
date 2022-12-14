import ReconnectingWebSocket from 'reconnecting-websocket';

type OnMessageCallback = (ev: MessageEvent<any>) => any;
type OnOpenCallback = () => void;
type OnCloseCallback = () => void;

let socket: ReconnectingWebSocket = null;

const connect = (onMessageCb: OnMessageCallback, onOpenCb: OnOpenCallback, onCloseCb: OnCloseCallback) =>  {
    doDisconnect()

    const { location } = window

    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws'
    const wsUri = `${proto}://${location.host}/session/ws`

    socket = new ReconnectingWebSocket(wsUri)

    socket.onopen = () => {
      onOpenCb()
    }

    socket.onmessage = onMessageCb

    socket.onclose = () => {
      onCloseCb()
    }
}

const doDisconnect = () => {
    if (socket) {
      socket.close()
      socket = null
    }
}

const doSend = (msg: string) => {
    if (socket) {
        socket.send(msg)
    }
}

const useWebSocket = (onMessageCb: OnMessageCallback) => {
  const doConnect = connect.bind(null, onMessageCb)
  return { doConnect, doDisconnect, doSend }
}

export default useWebSocket