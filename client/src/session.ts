import useWebSocket from "./websocket"
import "./css/base.css"

const { connect, send } = useWebSocket()

connect()

const searchInput = document.querySelector<HTMLInputElement>("#search-input")

const searchButton = document.querySelector<HTMLButtonElement>("#search-btn")
searchButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    const input = searchInput.value
    console.log("Search with search field input: ", input)
    searchInput.value = ""
})

const wsPingButton = document.querySelector<HTMLButtonElement>("#ws-ping")
wsPingButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    send("Ping sent from some peer...")
})

