import useWebSocket from "./websocket"
import "./css/base.css"

const { connect, send } = useWebSocket()

connect()

const wsPingButton = document.querySelector<HTMLButtonElement>("#ws-ping")
wsPingButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    send("Ping sent from some peer...")
})

