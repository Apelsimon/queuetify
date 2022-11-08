import useWebSocket from "./websocket"
import "./css/base.css"

interface TrackInfo {
    name: string;
    artists: string[];
    id: string;
}

interface SearchResults {
    tracks: TrackInfo[];
}

interface StateUpdate {
    track: TrackInfo | null;
    queue: TrackInfo[];
}

const onMessageCb = (ev: MessageEvent<any>) => {
    let result = JSON.parse(ev.data)

    switch (result.type) {
        case "SearchResult": {
            let searchResults = result.payload as SearchResults
            searchResultsList.textContent = ""
            searchResultsList.appendChild(createTrackList(searchResults.tracks))
            break;
        }
        case "StateUpdate": {
            let stateUpdate = result.payload as StateUpdate
            trackQueue.textContent = ""

            if (stateUpdate.track) {
                var paragraph = document.createElement("p")
                paragraph.textContent = "Current track: " + stateUpdate.track.name + " - " + stateUpdate.track.artists
                trackQueue.appendChild(paragraph)
            }
            
            trackQueue.appendChild(createTrackList(stateUpdate.queue))
            console.log("Receive state update: ", result.payload)
            break;
        }
    }
}

const { doConnect, doSend } = useWebSocket(onMessageCb)

const onOpenCb = () => {
    console.log("Fetch state!")
    const stateRequest = { type: "State" }
    doSend(JSON.stringify(stateRequest))
}

doConnect(onOpenCb)


const createTrackListEntry = (info: TrackInfo) => {
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

const createTrackList = (tracks: TrackInfo[]) => {
    var container = document.createElement("ul")
    
    for (var track of tracks) {
        container.appendChild(createTrackListEntry(track))
    }

    return container
}

const searchResultsList = document.querySelector<HTMLDivElement>("#search-results")
const trackQueue = document.querySelector<HTMLDivElement>("#track-queue")
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

