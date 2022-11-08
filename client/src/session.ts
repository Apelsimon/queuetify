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
            searchResultsList.appendChild(createTrackList(searchResults.tracks, "Add", queueTrack))
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
            
            trackQueue.appendChild(createTrackList(stateUpdate.queue, "Vote", voteTrack))
            console.log("Receive state update: ", result.payload)
            break;
        }
    }
}

const { doConnect, doSend } = useWebSocket(onMessageCb)

const onOpenCb = () => {
    const stateRequest = { type: "State" }
    doSend(JSON.stringify(stateRequest))
}

doConnect(onOpenCb)


const queueTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Queue track ", trackId)
    const queueRequest = { type: "Queue", uri: trackId }
    doSend(JSON.stringify(queueRequest))
}

const voteTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Vote for track ", trackId)
    const voteRequets = { type: "Vote", uri: trackId }
    doSend(JSON.stringify(voteRequets))
}

const createTrackListEntry = (info: TrackInfo, buttonText: string, onClickCb: (ev: MouseEvent, trackId: string) => void) => {
    var listEntry = document.createElement("li")
    
    var paragraph = document.createElement("p")
    paragraph.textContent = info.name + " - " + info.artists
    listEntry.appendChild(paragraph)

    const callback = (ev: MouseEvent, trackId: string) => {
        ev.preventDefault()
        onClickCb(ev, trackId)
    }
    const swap = function (trackId: string, ev: MouseEvent) {
        return this(ev, trackId);
    }
    var button = document.createElement("button")
    button.innerText = buttonText
    button.addEventListener("click", swap.bind(callback, info.id))
    listEntry.appendChild(button)

    return listEntry
}

const createTrackList = (tracks: TrackInfo[], buttonText: string, onClickCb: (ev: MouseEvent, trackId: string) => void) => {
    var container = document.createElement("ul")
    
    for (var track of tracks) {
        container.appendChild(createTrackListEntry(track, buttonText, onClickCb))
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

