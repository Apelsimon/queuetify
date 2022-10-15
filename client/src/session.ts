import useWebSocket from "./websocket"
import axios from 'axios'
import "./css/base.css"

const { connect, send } = useWebSocket()

connect()

const searchInput = document.querySelector<HTMLInputElement>("#search-input")

const searchButton = document.querySelector<HTMLButtonElement>("#search-btn")
searchButton.addEventListener("click", async (ev) => {
    ev.preventDefault()

    const input = searchInput.value
    searchInput.value = ""

    try {
        let result = await axios.get("/session/search?input=".concat(input));
        console.log("Received: ", JSON.stringify(result.data))
    } catch (error) {
        console.log("Error on search endpoint: ", error);
    }
})

const wsPingButton = document.querySelector<HTMLButtonElement>("#ws-ping")
wsPingButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    send("Ping sent from some peer...")
})

