import useWebSocket from "./websocket"
import "./css/base.css"

const onMessageCb = (ev: MessageEvent<any>) => {
    let result = JSON.parse(ev.data)

    switch (result.type) {
        case "SearchResult": {
            let searchResults = result.payload as SearchResults
            searchResultsList.textContent = ""
            searchResultsList.appendChild(createSearchResultList(searchResults))
        }
    }
}

const { doConnect, doSend } = useWebSocket(onMessageCb)
doConnect()

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
        const queueRequest = { type: "Queue", uri: trackId }
        doSend(JSON.stringify(queueRequest))
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
searchButton.addEventListener("click", (ev) => {
    ev.preventDefault()

    if (!searchInput.value) return

    const input = searchInput.value
    searchInput.value = ""

    const searchRequest = { type: "Search", query: input }
    doSend(JSON.stringify(searchRequest))
})

const wsPingButton = document.querySelector<HTMLButtonElement>("#ws-ping")
wsPingButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    doSend("Ping sent from some peer...")
})

