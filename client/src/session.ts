import useWebSocket from "./websocket"
import axios from 'axios'
import "./css/base.css"
import { fromJSON } from "postcss"

const { connect, send } = useWebSocket()
connect()

interface TrackInfo {
    name: string;
    artists: string[];
    id: string;
}

interface SearchResults {
    tracks: TrackInfo[];
}

const createSearchResultListEntry = (info: TrackInfo) => {
    var listEntry = document.createElement("li")
    
    var paragraph = document.createElement("p")
    paragraph.textContent = info.name + " - " + info.artists
    listEntry.appendChild(paragraph)

    const callback = (ev: MouseEvent, trackId: string) => {
        ev.preventDefault()
        console.log("callback called with id: ", trackId)
    }
    const swap = function (trackId: string, ev: MouseEvent) {
        return this(ev, trackId);
    }
    var addButton = document.createElement("button")
    addButton.innerText = "Add"
    addButton.addEventListener("click", swap.bind(callback, info.id))
    listEntry.appendChild(addButton)

    return listEntry
}

const createSearchResultList = (results: SearchResults) => {
    var container = document.createElement("ul")
    
    for (var track of results.tracks) {
        container.appendChild(createSearchResultListEntry(track))
    }

    return container
}

const searchResultsList = document.querySelector<HTMLDivElement>("#search-results")
const searchInput = document.querySelector<HTMLInputElement>("#search-input")

const searchButton = document.querySelector<HTMLButtonElement>("#search-btn")
searchButton.addEventListener("click", async (ev) => {
    ev.preventDefault()

    const input = searchInput.value
    searchInput.value = ""

    try {
        let result = await axios.get("/session/search?input=".concat(input));
        let searchResults = result.data as SearchResults
        searchResultsList.textContent = ""
        searchResultsList.appendChild(createSearchResultList(searchResults))
        
    } catch (error) {
        console.log("Error on search endpoint: ", error);
    }
})

const wsPingButton = document.querySelector<HTMLButtonElement>("#ws-ping")
wsPingButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    send("Ping sent from some peer...")
})

