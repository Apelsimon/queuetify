import useWebSocket from "./websocket"
import "./css/base.css"
import "./css/session.css"

import '@fortawesome/fontawesome-free/js/fontawesome'
import '@fortawesome/fontawesome-free/js/solid'
import '@fortawesome/fontawesome-free/js/regular'
import '@fortawesome/fontawesome-free/js/brands'


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

enum Context {
    Host,
    Peer,
    None
}

let context = Context.None

switch(document.querySelector<HTMLParagraphElement>("#context").innerText) {
    case "host": {
        context = Context.Host
        break
    }
    case "peer": {
        context = Context.Peer
        break
    }
}

const logout = () => {
    window.location.href = "/session/logout"
}

const onMessageCb = (ev: MessageEvent<any>) => {
    let result = JSON.parse(ev.data)

    switch (result.type) {
        case "Devices": {
            console.log("got devices: ", result.payload)
            break
        }
        case "SearchResult": {
            let searchResults = result.payload as SearchResults
            searchResultsList.textContent = ""
            searchResultsList.appendChild(createTrackList(searchResults.tracks, "Add", queueTrack))
            break
        }
        case "StateUpdate": {
            let stateUpdate = result.payload as StateUpdate
            trackQueue.textContent = ""

            if (stateUpdate.track) {
                let currentTrackContainer = document.createElement("div")
                currentTrackContainer.id = "current-track-container"
                let paragraph = document.createElement("p")
                paragraph.textContent = stateUpdate.track.name + " - " + stateUpdate.track.artists
                currentTrackContainer.appendChild(paragraph)

                let volumeIcon = document.createElement("i")
                volumeIcon.classList.add("fa")
                volumeIcon.classList.add("fa-volume-up")
                volumeIcon.ariaHidden = "true"
                volumeIcon.id = "volume-icon"
                currentTrackContainer.appendChild(volumeIcon)

                trackQueue.appendChild(currentTrackContainer)
            }
            
            trackQueue.appendChild(createTrackList(stateUpdate.queue, "Vote", voteTrack))
            console.log("Receive state update: ", result.payload)
            break
        }
        case "Shutdown": {
            logout()
        }
    }
}

const { doConnect, doSend } = useWebSocket(onMessageCb)

const onOpenCb = () => {
    const devicesRequest = { type: "Devices" }
    doSend(JSON.stringify(devicesRequest))

    const stateRequest = { type: "State" }
    doSend(JSON.stringify(stateRequest))
}

doConnect(onOpenCb)

const settingsNav = document.querySelector<HTMLDivElement>("#settings-nav")
const settingsNavContent = document.querySelector<HTMLDivElement>("#settings-nav-content")

const closeIcon = document.createElement("i")
closeIcon.classList.add("fa")
closeIcon.classList.add("fa-times")
closeIcon.ariaHidden = "true"

const closeSettingsNavButton = document.querySelector<HTMLButtonElement>("#close-settings-nav-btn")
closeSettingsNavButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    settingsNav.style.width = "0" 
})

if (context === Context.Host) {
    const endSessionButton = document.createElement("button")
    endSessionButton.innerText = "End session"
    endSessionButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        const killRequest = { type: "Kill"}
        doSend(JSON.stringify(killRequest))
    })
    settingsNavContent.appendChild(endSessionButton)

    const copyJoinUrlButton = document.createElement("button")
    copyJoinUrlButton.innerText = "Copy URL"
    copyJoinUrlButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        const sessionId = document.querySelector<HTMLButtonElement>("#session_id")
        const { location } = window
        const url = `${location.protocol}//${location.host}/join/${sessionId.innerText}`
        console.log("Copy join url ", url)
        
        navigator.clipboard.writeText(url)
    })
    settingsNavContent.appendChild(copyJoinUrlButton)

} else if (context === Context.Peer) {
    const exitSessionButton = document.createElement("button")
    exitSessionButton.innerText = "Exit session"
    exitSessionButton.addEventListener("click", (ev) => {
        ev.preventDefault()
        logout()
    })
    settingsNavContent.appendChild(exitSessionButton)
}

const queueTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Queue track ", trackId)
    const queueRequest = { type: "Queue", uri: trackId }
    doSend(JSON.stringify(queueRequest))
    closeSearchNavButton.click()
}

const voteTrack = (ev: MouseEvent, trackId: string) => {
    console.log("Vote for track ", trackId)
    const voteRequets = { type: "Vote", uri: trackId }
    doSend(JSON.stringify(voteRequets))
}

const createTrackListEntry = (info: TrackInfo, buttonText: string, onClickCb: (ev: MouseEvent, trackId: string) => void) => {

    var listEntry = document.createElement("li")
    listEntry.className = "track-container"
    
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
const searchNav = document.querySelector<HTMLButtonElement>("#search-nav")

const searchToggle = document.querySelector<HTMLButtonElement>("#search-toggle")
searchToggle.addEventListener("click", (ev) => {
    ev.preventDefault()
    searchNav.style.height = "100%"
})

const searchButton = document.querySelector<HTMLButtonElement>("#search-btn")
searchButton.addEventListener("click", (ev) => {
    ev.preventDefault()

    if (!searchInput.value) return

    const input = searchInput.value
    searchInput.value = ""

    const searchRequest = { type: "Search", query: input }
    doSend(JSON.stringify(searchRequest))
})

const closeSearchNavButton = document.querySelector<HTMLButtonElement>("#close-search-nav-btn")
closeSearchNavButton.addEventListener("click", (ev) => {
    ev.preventDefault()
    searchNav.style.height = "0"
})

const settingsToggle = document.querySelector<HTMLButtonElement>("#settings-toggle")
settingsToggle.addEventListener("click", (ev) => {
    ev.preventDefault()
    const settingsNav = document.querySelector<HTMLButtonElement>("#settings-nav")
    settingsNav.style.width = "100%"
})